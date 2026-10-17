use super::*;

impl WrappedText {
    pub(crate) fn new(position: Position, format: TextFormat, resources: &RenderContext) -> Self {
        Self {
            source: Source::whole(position.block, resources.cell),
            position,
            format,
            resources: resources.clone(),
            seed: None,
        }
    }

    pub(crate) fn new_span(
        mother_position: Position,
        span: (usize, usize),
        format: TextFormat,
        resources: &RenderContext,
        index: Arc<InlineIndex>,
    ) -> Self {
        Self {
            source: Source {
                span: Some(span),
                ..Source::whole(mother_position.block, resources.cell)
            },
            position: mother_position,
            format,
            resources: resources.clone(),
            seed: Some(index),
        }
    }
}
