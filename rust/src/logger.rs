use std::fmt::Write;

use godot::global::godot_print_rich;
use tracing::{field::Visit, level_filters::LevelFilter, span, Subscriber};

pub struct Logger {
    pub max_level: LevelFilter,
}
#[derive(Default)]
struct LoggerVisitor {
    rest: String,
    message: String,
}

impl Visit for LoggerVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            write!(&mut self.message, "{:?}", value).unwrap();
        } else {
            write!(&mut self.rest, "{}: {:?},", field.name(), value).unwrap();
        }
    }
}

impl Subscriber for Logger {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.level() <= &self.max_level
    }

    fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(0)
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let level = event.metadata().level();
        if level > &self.max_level {
            return;
        }
        let col = match *level {
            tracing::Level::ERROR => "red",
            tracing::Level::WARN => "yellow",
            tracing::Level::INFO => "green",
            tracing::Level::DEBUG => "blue",
            tracing::Level::TRACE => "purple",
        };
        let prefix = level.as_str().to_uppercase();
        let pre = format!("[color={}]{}[/color]", col, prefix);

        let mut visitor = LoggerVisitor::default();
        event.record(&mut visitor);
        visitor.rest.pop();
        let target = match event.metadata().module_path() {
            Some(m) => format!("[code]{}[/code]", m),
            None => "".to_string(),
        };
        let line = match event.metadata().line() {
            Some(l) => format!("<[code]{}[/code]>", l),
            None => "".to_string(),
        };

        godot_print_rich!(
            "{}{} {}: [{}]: {}",
            target,
            line,
            pre,
            visitor.rest,
            visitor.message
        );
    }

    fn enter(&self, _span: &span::Id) {}

    fn exit(&self, _span: &span::Id) {}
}
