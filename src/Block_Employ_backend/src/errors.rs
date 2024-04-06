#[derive(thiserror::Error, Debug)]
pub enum Error{
    #[Error("youve not applied")]
    XValueNotOfType(&'static str),

    #[Error(transparent)]
    Surreal(#[from]surrealdb::Error),

    #[Error(transparent)]
    IO(#[from] std::io::Error),
}