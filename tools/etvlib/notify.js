/* A notice at the foot of the window instead of `alert()`.
 *
 * `alert()` stops the whole window until it is clicked away and looks like
 * the system, not like the app. `notify(text)` shows a note that leaves by
 * itself; `notify(text, true)` an error, which stays until it is closed —
 * an error that vanishes while one looks elsewhere was never said.
 *
 * Not for a modal dialog: that lies above everything else in the page, a
 * notice would sit unseen behind it. A dialog says its errors itself.
 *
 * Nothing runs on import.
 */

//: How long a note that is no error stays, in milliseconds.
const STAY_MS = 6000;

function shelf(){
  let el = document.getElementById("notices");
  if (!el){
    el = Object.assign(document.createElement("div"), { id: "notices", className: "notices" });
    document.body.append(el);
  }
  return el;
}

//: Show *text*; as an error when *isError*. Returns the notice's element.
export function notify(text, isError = false){
  const note = document.createElement("div");
  note.className = "banner notice" + (isError ? " danger" : "");
  // An error interrupts a screen reader, a note waits its turn.
  note.setAttribute("role", isError ? "alert" : "status");
  const words = Object.assign(document.createElement("div"), { className: "grow", textContent: text });
  const close = Object.assign(document.createElement("button"), { className: "btn small ghost", textContent: "×" });
  close.setAttribute("aria-label", "Dismiss the message");
  close.onclick = () => note.remove();
  note.append(words, close);
  // The same sentence twice says nothing new: the newer one replaces it.
  for (const old of shelf().children) if (old.firstChild.textContent === text) old.remove();
  shelf().append(note);
  if (!isError) setTimeout(() => note.remove(), STAY_MS);
  return note;
}
