use nitro::SieveCache;

#[test]
fn test_basic_integration() {
    let mut cache: SieveCache<String, i32> = SieveCache::new(2).unwrap();

    // Test basic operations in sequence
    assert_eq!(cache.add(String::from("key1"), 1).unwrap(), false);
    assert_eq!(cache.get(&String::from("key1")).unwrap(), Some(1));

    assert_eq!(cache.add(String::from("key2"), 2).unwrap(), false);
    assert_eq!(cache.get(&String::from("key2")).unwrap(), Some(2));

    // This should trigger eviction
    assert_eq!(cache.add(String::from("key3"), 3).unwrap(), false);

    // key1 should be evicted as it wasn't accessed
    assert_eq!(cache.get(&String::from("key1")).unwrap(), None);
}

#[test]
fn test_complex_operations() {
    let mut cache: SieveCache<String, String> = SieveCache::new(3).unwrap();

    // Add items
    cache.add(String::from("a"), String::from("alpha")).unwrap();
    cache.add(String::from("b"), String::from("beta")).unwrap();

    // Access an item
    assert_eq!(
        cache.get(&String::from("a")).unwrap(),
        Some(String::from("alpha"))
    );

    // Probe
    let (val, exists) = cache
        .probe(String::from("c"), String::from("gamma"))
        .unwrap();
    assert_eq!(val, String::from("gamma"));
    assert_eq!(exists, false);

    // Delete
    assert!(cache.delete(&String::from("b")).unwrap());
    assert_eq!(cache.get(&String::from("b")).unwrap(), None);

    // Check stats
    let stats = cache.get_stats();
    assert!(stats.hits > 0);
    assert!(stats.misses > 0);
}
