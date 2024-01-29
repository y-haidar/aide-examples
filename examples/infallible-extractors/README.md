# Infallible Extractors

Showcase a workflow where all extractors are infallible, but returning a `Result<Result<T, E>, infallible>`. Also all routes docs are handles by the `TransformOperation`. Reasoning behind this, is to limit the side effect of using `OperationInput` and `OperationOutput`, and have a single place to control both the actual implementation of the route and the documentation of its responses.
