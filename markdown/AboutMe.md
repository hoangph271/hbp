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

---
![blur:5px bg](https://img.freepik.com/free-vector/abstract-texture-background_91008-369.jpg)

## > Marp
This slide show is powered my `Marpit` - *the open source Markdown slide deck framework*  

You can find the `npm` package (`@marp-team/marp-core`) [here](https://www.npmjs.com/package/@marp-team/marp-core)...!  

It can do cool things, like:
 - [x] Change the background...?
 - [x] Blur the background by 5px...!
 - [x] And link to [my GitHub](https://github.com/hoangph271)...!
