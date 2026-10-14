use custom_iterators::CustomIteratorExt;
use std::cell::Cell;

#[test]
fn test_my_map_basic() {
    let numbers = vec![1, 2, 3, 4, 5];
    let mapped: Vec<i32> = numbers.into_iter().my_map(|x| x * 2).collect();
    assert_eq!(mapped, vec![2, 4, 6, 8, 10]);
}

#[test]
fn test_my_map_type_transformation() {
    let numbers = vec![1, 2, 3];
    let mapped: Vec<String> = numbers.into_iter().my_map(|x| format!("num: {}", x)).collect();
    assert_eq!(mapped, vec!["num: 1", "num: 2", "num: 3"]);
}

#[test]
fn test_my_filter_basic() {
    let numbers = vec![1, 2, 3, 4, 5, 6];
    let evens: Vec<i32> = numbers.into_iter().my_filter(|x| x % 2 == 0).collect();
    assert_eq!(evens, vec![2, 4, 6]);
}

#[test]
fn test_chaining_filter_and_map() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result: Vec<i32> = numbers
        .into_iter()
        .my_filter(|x| x % 2 != 0) // [1, 3, 5, 7, 9]
        .my_map(|x| x * 10)         // [10, 30, 50, 70, 90]
        .my_filter(|x| *x > 40)    // [50, 70, 90]
        .collect();

    assert_eq!(result, vec![50, 70, 90]);
}

#[test]
fn test_laziness_evaluation() {
    let map_eval_count = Cell::new(0);
    let filter_eval_count = Cell::new(0);

    let numbers = vec![1, 2, 3, 4, 5];

    // Build iterator pipeline without consuming
    let mut pipeline = numbers
        .into_iter()
        .my_filter(|x| {
            filter_eval_count.set(filter_eval_count.get() + 1);
            x % 2 == 0
        })
        .my_map(|x| {
            map_eval_count.set(map_eval_count.get() + 1);
            x * 10
        });

    // Verify 0 executions before polling next()
    assert_eq!(filter_eval_count.get(), 0);
    assert_eq!(map_eval_count.get(), 0);

    // Poll next() once:
    // It should test 1 (false), test 2 (true), then map 2 -> 20.
    let first = pipeline.next();
    assert_eq!(first, Some(20));
    assert_eq!(filter_eval_count.get(), 2);
    assert_eq!(map_eval_count.get(), 1);

    // Poll next() second time:
    // It should test 3 (false), test 4 (true), then map 4 -> 40.
    let second = pipeline.next();
    assert_eq!(second, Some(40));
    assert_eq!(filter_eval_count.get(), 4);
    assert_eq!(map_eval_count.get(), 2);

    // Poll next() third time:
    // It should test 5 (false), exhausted -> None.
    let third = pipeline.next();
    assert_eq!(third, None);
    assert_eq!(filter_eval_count.get(), 5);
    assert_eq!(map_eval_count.get(), 2);

    // Further calls stay None and don't re-execute map
    assert_eq!(pipeline.next(), None);
    assert_eq!(map_eval_count.get(), 2);
}

#[test]
fn test_empty_iterator() {
    let empty: Vec<i32> = vec![];
    let result: Vec<i32> = empty
        .into_iter()
        .my_map(|x| x * 2)
        .my_filter(|x| *x > 0)
        .collect();

    assert!(result.is_empty());
}

#[test]
fn test_size_hint() {
    let numbers = vec![1, 2, 3, 4];
    let mapped = numbers.iter().my_map(|x| x * 2);
    assert_eq!(mapped.size_hint(), (4, Some(4)));

    let filtered = numbers.iter().my_filter(|x| **x > 2);
    assert_eq!(filtered.size_hint(), (0, Some(4)));
}
