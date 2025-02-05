fn main() {
    match ios::retriever() {
        Ok(_) => {
            println!("Backup has been extracted");
        }
        Err(err) => {
            println!("{}", err);
        }
    };
}
