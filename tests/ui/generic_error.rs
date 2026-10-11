use smarterr::errorset::errorset;

#[errorset(export)]
fn generic<T>() -> Result<(), (T,)> {
    Ok(())
}

fn main() {}
