use nitro::SieveCache;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache: SieveCache<String, String> = SieveCache::new(3)?;

    // Add some items
    println!("Adding initial items...");
    cache
        .add(String::from("key1"), String::from("value1"))
        .unwrap();
    cache
        .add(String::from("key2"), String::from("value2"))
        .unwrap();
    cache
        .add(String::from("key3"), String::from("value3"))
        .unwrap();

    // Get an item (marks it as visited)
    println!("\nTrying to get key1...");
    if let Some(value) = cache.get(&String::from("key1")).unwrap() {
        println!("Found value for key1: {}", value);
    }

    // Add another item, which should trigger eviction
    println!("\nAdding key4 (should trigger eviction)...");
    cache
        .add(String::from("key4"), String::from("value4"))
        .unwrap();

    // Try to get potentially evicted items
    println!("\nChecking which keys are still in cache:");
    for key in ["key1", "key2", "key3", "key4"].iter() {
        match cache.get(&String::from(*key)).unwrap() {
            Some(value) => println!("{}: {} (still in cache)", key, value),
            None => println!("{}: was evicted", key),
        }
    }
    Ok(())
}
