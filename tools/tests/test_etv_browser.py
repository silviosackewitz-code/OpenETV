"""A click-through of the window in WebKit — the engine pywebview uses on macOS.

Real keys, a real file chooser, a real download: what the unit tests of the
modules cannot see. Needs Playwright and its WebKit
(`.venv/bin/python -m playwright install webkit`); skipped without them.

Every test gets a fresh page and fails if the page reports an error.
"""

import json

import pytest

from etvlib import dss, server, tables
from etvlib.tables import make_table

playwright_api = pytest.importorskip("playwright.sync_api")
expect = playwright_api.expect


@pytest.fixture(scope="module")
def url():
    httpd, address = server.serve()
    yield address
    httpd.shutdown()


@pytest.fixture(scope="module")
def browser():
    with playwright_api.sync_playwright() as p:
        try:
            engine = p.webkit.launch()
        except playwright_api.Error as exc:
            pytest.skip(f"WebKit for Playwright is not installed: {exc}")
        yield engine
        engine.close()


@pytest.fixture
def page(browser, url):
    context = browser.new_context(viewport={"width": 1280, "height": 820}, accept_downloads=True)
    errors = []
    pg = context.new_page()
    pg.on("pageerror", lambda exc: errors.append(str(exc)))
    pg.on("console", lambda msg: msg.type == "error" and errors.append(msg.text))
    pg.goto(url)
    expect(pg.locator('section[data-step="engine"] .grid td')).to_have_count(77)     # the sample: 7 × 11
    yield pg
    context.close()
    assert errors == []


def step(pg, name):
    pg.locator(f'#steps [data-step="{name}"]').click()
    return pg.locator(f'main > section[data-step="{name}"]')


def line(pg, name):
    return pg.locator(f'#steps [data-step="{name}"] small')


def calculate(pg):
    step(pg, "calculation")
    pg.get_by_role("button", name="Calculate ETV map").click()
    expect(pg.locator('#steps [aria-selected="true"]')).to_have_attribute("data-step", "result")


def test_the_window_starts_with_the_sample_tables_and_says_how_things_stand(page):
    expect(page.locator("#version")).not_to_be_empty()
    expect(line(page, "engine")).to_have_text("7 × 11, up to 98 Nm")
    expect(line(page, "result")).to_have_text("not calculated")
    expect(page.locator('main > section:not([hidden])')).to_have_count(1)
    expect(page.locator("#resultBlank")).to_be_hidden()                # another step is shown
    step(page, "result")
    expect(page.locator("#resultBlank")).to_be_visible()
    for fix in page.locator("[data-fix]").all():
        expect(fix).to_be_disabled()


def test_a_request_is_generated_and_its_curve_is_drawn(page):
    section = step(page, "request")
    section.locator("#generator > summary").click()
    expect(section.locator("#genCenter")).to_be_hidden()
    section.locator("#genShape").select_option("s_curve")
    expect(section.locator("#genCenter")).to_be_visible()
    # Something is drawn on the scope: the curve's bright pixels.
    page.wait_for_function("""() => { const c = document.getElementById('genCurve');
        const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data; let lit = 0;
        for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 380) lit++; return lit > 500; }""")
    section.locator("#genShape").select_option("linear")
    section.get_by_role("button", name="Generate request").click()
    expect(line(page, "request")).to_have_text("7 × 24, up to 98 Nm")
    # Absolute torque: 3000 rpm has 55 Nm at most, and the request is capped there.
    expect(section.locator('.grid td[data-r="1"]').last).to_have_text("55")


def test_the_map_is_calculated_and_its_cells_say_what_happened(page):
    calculate(page)
    expect(line(page, "result")).to_have_text("7 of 77 cells saturated")
    expect(page.locator('#resultGrid td[data-mark="saturated"]')).to_have_count(7)
    expect(page.locator("#resultLegend")).to_be_visible()
    expect(page.locator("#resultNotes .banner")).to_contain_text("7 cell(s) are saturated")
    expect(page.locator('#resultGrid td[data-r="1"][data-c="1"]')).to_have_text("4.1")


def test_typing_into_a_table_changes_it_and_makes_the_map_out_of_date(page):
    calculate(page)
    section = step(page, "engine")
    cell = section.locator('.grid td[data-r="1"][data-c="11"]')
    cell.click()
    page.keyboard.type("120")                                         # the click selected the old number
    expect(line(page, "engine")).to_have_text("7 × 11, up to 120 Nm")  # taken while typed
    page.keyboard.press("Enter")
    expect(section.locator('.grid td[data-r="2"][data-c="11"]')).to_be_focused()

    expect(line(page, "result")).to_have_text("out of date — calculate again")
    expect(line(page, "result")).to_have_class("warn")
    result = step(page, "result")
    expect(result).to_have_attribute("data-stale", "1")
    expect(page.locator("#resultNotes .banner").first).to_contain_text("changed since this map was calculated")
    calculate(page)
    expect(step(page, "result")).to_have_attribute("data-stale", "0")


def test_a_breakpoint_moves_its_column_when_its_cell_is_left(page):
    section = step(page, "engine")
    section.locator('.grid th[data-r="0"][data-c="2"]').click()        # throttle 10 %
    page.keyboard.type("95")
    page.keyboard.press("Tab")
    headers = section.locator(".grid thead th[data-c]")
    expect(headers.nth(9)).to_have_text("95")                         # between 90 and 100 now
    expect(headers.nth(10)).to_have_text("100")
    expect(section.locator('.grid td[data-r="1"][data-c="10"]')).to_have_text("9.6")   # its values went with it


def test_what_is_no_number_is_marked_and_not_calculated_with(page):
    section = step(page, "engine")
    section.locator('.grid td[data-r="1"][data-c="3"]').click()
    page.keyboard.type("abc")
    page.keyboard.press("Tab")
    expect(section.locator('.grid td[data-r="1"][data-c="3"]')).to_have_class("bad")
    expect(line(page, "engine")).to_have_text("7 × 11, up to 98 Nm")   # the table still holds the old number


def test_rows_and_columns_are_added_and_removed(page):
    section = step(page, "engine")
    section.get_by_role("button", name="Row", exact=True).click()
    section.get_by_role("button", name="Column", exact=True).click()
    expect(line(page, "engine")).to_have_text("8 × 12, up to 98 Nm")
    expect(section.locator(".grid tbody th").last).to_have_text("17000")   # one more step of 2000
    section.locator('.grid td[data-r="8"][data-c="12"]').click()
    section.get_by_role("button", name="− Row").click()
    expect(line(page, "engine")).to_have_text("7 × 12, up to 98 Nm")


def test_the_zero_gas_fix_is_applied_and_says_how_much_it_changed(page):
    calculate(page)
    page.get_by_role("button", name="Zero-gas fix").click()
    expect(page.locator(".notice")).to_contain_text("cell(s) changed")
    for cell in page.locator('#resultGrid td[data-c="1"]').all():
        expect(cell).to_have_text("0")
    page.get_by_role("button", name="Remove dips").click()
    expect(page.locator(".notice").last).to_have_text("Nothing to change.×")


def test_a_file_is_loaded_and_a_dss_says_which_of_its_tables_cannot_be_used(page, tmp_path):
    csv = tmp_path / "engine.csv"
    csv.write_text("RPM;0;100\n4000;0;40,5\n8000;0;81\n", encoding="utf-8")
    section = step(page, "engine")
    with page.expect_file_chooser() as chooser:
        section.get_by_role("button", name="Load file …").click()
    chooser.value.set_files(str(csv))
    expect(line(page, "engine")).to_have_text("2 × 2, up to 81 Nm")

    # A real ECU export named "demand" holds throttle in %: refused, with the reason.
    etv = make_table([4000, 8000], [0, 100], [[0, 50], [0, 100]])
    file = tmp_path / "demand.dss"
    file.write_text(dss.build_dss_xml(etv, "Demand.Dry.Gear1", "%", 0, 100, "BreakPt.RPM", "1/min", "BreakPt.GAS", "%"))
    section = step(page, "request")
    with page.expect_file_chooser() as chooser:
        section.get_by_role("button", name="Load file …").click()
    chooser.value.set_files(str(file))
    dialog = page.locator("#pick")
    expect(dialog).to_be_visible()
    expect(dialog.locator(".refused")).to_contain_text("throttle map")
    expect(dialog.locator("[data-pick]")).to_have_count(0)
    dialog.get_by_role("button", name="Close").click()
    expect(line(page, "request")).to_have_text("7 × 11, up to 98 Nm")  # the request is as it was


def test_in_a_browser_the_map_is_handed_to_the_downloads(page):
    calculate(page)
    step(page, "export")
    with page.expect_download() as download:
        page.get_by_role("button", name="Save as CSV …").click()
    assert download.value.suggested_filename == "etv_map_throttle.csv"
    saved = tables.read_csv(open(download.value.path(), encoding="utf-8").read())
    assert saved.values[0][:3].tolist() == [4.1, 5.0, 6.7]
    expect(page.locator(".notice")).to_contain_text("handed to the browser's downloads")


def test_the_look_is_chosen_in_the_window_and_kept(page, own_prefs_file):
    page.get_by_role("button", name="Appearance").click()
    page.locator("#look .segmented label", has_text="Dark").click()
    expect(page.locator("html")).to_have_attribute("data-theme", "dark")
    page.locator('#look button.swatch[title="Blue"]').click()
    expect(page.locator('#look button.swatch[title="Blue"]')).to_have_attribute("aria-pressed", "true")
    assert json.loads(own_prefs_file.read_text()) == {"theme": "dark", "accent": "#0a5fd1"}
    page.locator("#look").get_by_role("button", name="Close").click()
    expect(page.locator("#look")).to_be_hidden()


def test_narrow_the_steps_lie_above_the_step_and_nothing_sticks_out(page):
    page.set_viewport_size({"width": 600, "height": 800})
    assert page.evaluate("document.documentElement.scrollWidth <= innerWidth")
    side, main = page.locator(".side").bounding_box(), page.locator("main").bounding_box()
    assert side["y"] + side["height"] <= main["y"] + 1
