# Native regression data

`numerical-baseline.jsonl` contains 375 distinct cases frozen before removing the former implementation at a8eacbc. Inputs come only from deterministic synthetic data (seed 20261026) and the bundled synthetic sample maps. Duplicate cases were removed. No private ECU exports or book material is included.

Each line holds an input batch operation and its independently computed expected result. These cases cover inversion, saturation, interpolation, generators, reverse lookup and corrections. Do not regenerate expected values from the code under test.

`theme-roles.json` preserves the shared Race Analysis light/dark palette and sizing constants for native theme tests.
