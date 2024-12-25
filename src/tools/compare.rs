use std::cmp::Ordering;

pub fn alphabetical_inc(a: &Vec<String>, b: &Vec<String>) -> std::cmp::Ordering {
    assert!(a.get(0).is_some() && b.get(0).is_some(), "Compared vectors have no zeroth element");
    a.get(0).unwrap().cmp(&b.get(0).unwrap())
}
