//! __app_name__ CLI scaffold.

fn main() {
    println!(
        "{} {} ({})",
        __app_ident___sdk::app(),
        env!("CARGO_PKG_VERSION"),
        __app_ident___sdk::slug()
    );
}
