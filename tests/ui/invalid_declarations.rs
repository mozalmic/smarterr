use smarterr::errorset::errorset;

#[errorset]
type NotTuple = reader::FileError;

#[errorset]
type Parenthesized = (reader::FileError);

#[errorset]
type Generic<T> = (T,);

#[errorset]
type Lifetime<'a> = (reader::FileError,);

#[errorset]
type ConstGeneric<const N: usize> = (reader::FileError,);

#[errorset]
type Duplicate = (reader::FileError, reader::VersionError, reader::FileError);

#[errorset(pub mod sets)]
type ModuleOption = (reader::FileError,);

#[errorset(from(SelfSource))]
type SelfSource = (reader::FileError,);

fn main() {}
