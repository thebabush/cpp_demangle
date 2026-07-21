//! Structured-event (`DemangleWrite`) coverage — pins the node boundaries a
//! structured consumer relies on. The rest of the suite only checks the rendered
//! string; this records the `push`/`text`/`pop` stream and asserts the markers.

use cpp_demangle::{DemangleNodeType, DemangleOptions, DemangleWrite, Symbol};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Push(DemangleNodeType),
    Text(String),
    Pop,
}

struct Recorder(Vec<Event>);

impl DemangleWrite for Recorder {
    fn push_demangle_node(&mut self, t: DemangleNodeType) {
        self.0.push(Event::Push(t));
    }
    fn write_string(&mut self, s: &str) -> fmt::Result {
        self.0.push(Event::Text(s.to_string()));
        Ok(())
    }
    fn pop_demangle_node(&mut self) {
        self.0.push(Event::Pop);
    }
}

fn events(mangled: &str) -> Vec<Event> {
    let sym = Symbol::new(mangled).expect("parse");
    let mut r = Recorder(Vec::new());
    sym.structured_demangle(&mut r, &DemangleOptions::new())
        .expect("demangle");
    r.0
}

/// The concatenated text of each `Push(node) … Pop` span that contains only text
/// (a node's `ensure_space()` + keyword are two separate writes, so a single-`Text`
/// match would miss ` const`).
fn marked_texts(evts: &[Event], node: DemangleNodeType) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < evts.len() {
        if evts[i] == Event::Push(node) {
            let mut s = String::new();
            let mut j = i + 1;
            let mut only_text = true;
            while j < evts.len() && evts[j] != Event::Pop {
                match &evts[j] {
                    Event::Text(t) => s.push_str(t),
                    _ => only_text = false,
                }
                j += 1;
            }
            if only_text && j < evts.len() {
                out.push(s);
            }
            i = j;
        }
        i += 1;
    }
    out
}

/// True if `text` is written inside its own `node` boundary (and nothing else).
fn has_marked(evts: &[Event], node: DemangleNodeType, text: &str) -> bool {
    marked_texts(evts, node).iter().any(|s| s == text)
}

#[test]
fn reference_is_its_own_node() {
    // f(int&)
    let e = events("_Z1fRi");
    assert!(has_marked(&e, DemangleNodeType::Reference, "&"), "{e:?}");
}

#[test]
fn rvalue_reference_is_its_own_node() {
    // f(int&&)
    let e = events("_Z1fOi");
    assert!(has_marked(&e, DemangleNodeType::Reference, "&&"), "{e:?}");
}

#[test]
fn pointer_and_cv_qualifier_are_their_own_nodes() {
    // f(char const*): a `CvQualifier` for " const" (space inside) then a `Pointer`.
    let e = events("_Z1fPKc");
    assert!(has_marked(&e, DemangleNodeType::CvQualifier, " const"), "{e:?}");
    assert!(has_marked(&e, DemangleNodeType::Pointer, "*"), "{e:?}");
}

#[test]
fn member_pointer_colon_star_is_a_pointer_node() {
    // A pointer-to-member: the `::*` declarator, with the member class as its own
    // (already-structured) subtree preceding it.
    let e = events("_ZN4llvm11IntervalMapINS_9SlotIndexEjLj9ENS_15IntervalMapInfoIS1_EEE10visitNodesEMS4_FvNS_15IntervalMapImpl7NodeRefEjE");
    assert!(has_marked(&e, DemangleNodeType::Pointer, "::*"), "{e:?}");
}

#[test]
fn cv_qualifier_keyword_is_its_own_node() {
    // A::f() const — the cv-qualifier is marked (` const`, space inside the node).
    // (The top-level ref-qualifier `&` in `A::f() &` is loose trailing text, a
    // different code path from a function-*type*'s tail — not marked here.)
    let e = events("_ZNK1A1fEv");
    assert!(has_marked(&e, DemangleNodeType::CvQualifier, " const"), "{e:?}");
}
