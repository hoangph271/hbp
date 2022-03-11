---
marp: true  
theme: gaia
---

## > HBP

> And just like that, the journey begins...!  
> Long, and exhausted...!  
> But they keep on walking...!  
> *- HBP, 2022*

```rust
#[launch]
async fn rocket() -> _ {
    dotenv::dotenv().ok();

    let app_name = utils::env::from_env(utils::env::EnvKey::AppName);
    println!("{app_name} is starting, my dude...! 🍿🍿🍿");
}
```
