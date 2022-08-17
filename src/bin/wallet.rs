use jab::wallet::Wallet;

fn main() {
    let wallet = Wallet::new();
    println!("your new wallet address: {}", wallet.address());
}
