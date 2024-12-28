pub fn increasing(a: &Vec<String>, b: &Vec<String>, id: usize) -> std::cmp::Ordering {
    assert!(a.get(id).is_some() && b.get(id).is_some(), "Compared vectors have no zeroth element");
    a.get(id).unwrap().cmp(&b.get(id).unwrap())
}


pub fn decreasing(a: &Vec<String>, b: &Vec<String>, id: usize) -> std::cmp::Ordering {
    assert!(a.get(id).is_some() && b.get(id).is_some(), "Compared vectors have no zeroth element");
    b.get(id).unwrap().cmp(&a.get(id).unwrap())
}
