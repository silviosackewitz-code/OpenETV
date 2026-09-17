import json

import pytest

from etvlib import api, prefs, theme


def test_without_a_file_the_defaults_apply(own_prefs_file):
    assert prefs.read_prefs() == {"theme": "auto", "accent": theme.DEFAULT_ACCENT}
    assert not own_prefs_file.exists()                                # reading writes nothing


def test_a_change_is_kept_and_the_rest_stays(own_prefs_file):
    assert prefs.set_prefs({"theme": "dark"}) == {"theme": "dark", "accent": theme.DEFAULT_ACCENT}
    assert prefs.set_prefs({"accent": "#0A5FD1"}) == {"theme": "dark", "accent": "#0a5fd1"}
    assert json.loads(own_prefs_file.read_text()) == {"theme": "dark", "accent": "#0a5fd1"}
    assert [p.name for p in own_prefs_file.parent.iterdir()] == ["prefs.json"]   # no temporary file left


@pytest.mark.parametrize("values, says", [({"theme": "blue"}, "must be one of auto, light, dark"),
                                          ({"accent": "red"}, "must be a colour"),
                                          ({"accent": "#d40000}body{display:none"}, "must be a colour"),
                                          ({"font": "Comic Sans"}, "unknown preference 'font'")])
def test_what_does_not_fit_is_refused_and_nothing_is_written(values, says, own_prefs_file):
    with pytest.raises(ValueError, match=says):
        prefs.set_prefs({"theme": "dark", **values})
    assert not own_prefs_file.exists()


@pytest.mark.parametrize("content", ["{broken", "[1, 2]", '{"theme": "blue", "accent": 7}'])
def test_a_file_that_cannot_be_used_reads_as_the_defaults(content, own_prefs_file):
    own_prefs_file.write_text(content)
    assert prefs.read_prefs() == {"theme": "auto", "accent": theme.DEFAULT_ACCENT}


def test_a_window_gets_the_rules_of_the_accent_and_the_colours_on_offer():
    assert api.get_prefs({})["accent_css"] == ""                      # the default needs no rules
    answer = api.post_prefs({"accent": "#0a5fd1"})
    assert answer["ok"] and answer["accent"] == "#0a5fd1" and answer["theme"] == "auto"
    assert answer["accent_css"].startswith(":root{--accent:")
    assert answer["accent_presets"][0] == {"name": "Red", "colour": theme.DEFAULT_ACCENT}
