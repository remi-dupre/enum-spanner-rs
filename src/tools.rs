use std::iter;

/// Return an iterator over the elements of a range that are not part if
/// elements of an input iterator.
///
/// Note: This function assumes that all elements are inside of the input
/// range.
pub fn iter_complement<U>(
    start: usize,
    end: usize,
    iterator: U,
) -> impl Clone + Iterator<Item = usize>
where
    U: Iterator<Item = usize>,
{
    let mut del_elements: Vec<_> = iterator.collect();
    del_elements.sort();
    del_elements
        .into_iter()
        .chain(iter::once(end))
        .scan(start, |expected_index, index| {
            let not_skiped = *expected_index..index;
            *expected_index = index + 1;
            Some(not_skiped)
        })
        .flatten()
}
