/// Returned by `Handler.on_headers_complete`, the power to instruct the parser
/// not to expect a body.
/// TODO: maybe a better way for *us* to do this would be to construct the
/// parser with the knowledge that it’s a HEAD response. I don’t really like
/// the way that joyent did it.
pub enum BodyExpectation {
    NoBody,
    MaybeBody,
}


