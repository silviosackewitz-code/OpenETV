"""The import graph of the window's modules, read from their sources.

The rule is the one of Race Analysis: a module that changes something says so
(`etv/events.js`) instead of calling whoever shows it, so imports only point at
what a module *uses* and the graph has no ring. With a ring, everything works
only as long as nothing runs on import — and every new area has to be wired
into every area it affects.
"""

import re
from pathlib import Path

ETV = Path(__file__).resolve().parents[1] / "etvlib" / "etv"
_IMPORT = re.compile(r'from\s+["\'](\./[^"\']+|/[^"\']+)["\']')

#: What Node has to be able to load by itself: no page, no server, no import.
PURE = {"tablemath.js", "ramp.js", "events.js", "state.js"}


def _graph() -> dict[str, set[str]]:
    return {src.name: {imp[2:] for imp in _IMPORT.findall(src.read_text(encoding="utf-8")) if imp.startswith("./")}
            for src in sorted(ETV.glob("*.js"))}


def test_every_import_names_a_module_that_exists():
    graph = _graph()
    assert len(graph) >= 15
    assert not {(src, imp) for src, imps in graph.items() for imp in imps if imp not in graph}


def test_the_graph_has_no_ring():
    graph, done, path = _graph(), set(), []

    def visit(name):
        if name in path:
            raise AssertionError("import ring: " + " → ".join([*path[path.index(name):], name]))
        if name in done:
            return
        path.append(name)
        for imp in sorted(graph[name]):
            visit(imp)
        path.pop()
        done.add(name)

    for name in graph:
        visit(name)


def test_the_pure_modules_import_nothing():
    for name in PURE:
        found = _IMPORT.findall((ETV / name).read_text(encoding="utf-8"))
        assert not found, f"{name} imports {found}"


def test_only_main_runs_something_when_it_is_loaded():
    # "Nothing runs on import": a module's top level defines, `init()` wires.
    for src in ETV.glob("*.js"):
        calls = re.findall(r"^(?!export|import|const|let|function|async function|//|/\*|\*| |\}|$)(\w[\w.]*)\(",
                           src.read_text(encoding="utf-8"), re.M)
        assert (src.name == "main.js") == bool(calls), (src.name, calls)
