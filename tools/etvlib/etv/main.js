// Placeholder of the window (docs/plan.md, step 5): one round through the API
// with the sample tables, so that a start shows at once whether server, key
// and calculation work together.

async function api(route, body) {
  const answer = await fetch(`/api/${route}`, body === undefined ? {} : {
    method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body),
  });
  const data = await answer.json();
  if (!data.ok) throw new Error(data.error);
  return data;
}

async function start() {
  const state = document.getElementById("state");
  try {
    const about = await api("about");
    const sample = await api("sample");
    const result = await api("calculate", { engine: sample.engine, request: sample.request });
    const { rpm, axis } = result.table;
    state.textContent = `Version ${about.version}. The sample tables give an ETV map of ` +
      `${rpm.length} × ${axis.length} cells, ${result.counts.saturated} of them saturated. ` +
      "The window itself is not built yet.";
  } catch (error) {
    state.textContent = `The server did not answer as expected: ${error.message}`;
  }
}

start();
