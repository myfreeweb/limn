use cassowary::strength::*;

use event::{Target, WidgetEventHandler, WidgetEventArgs};
use widget::{WidgetBuilder, WidgetBuilderCore};
use widget::style::Value;
use widgets::drag::{DragEvent, WidgetDrag};
use drawable::rect::{RectDrawable, RectStyleField};
use resources::WidgetId;
use util::Dimensions;

pub struct SliderBuilder {
    pub widget: WidgetBuilder,
}
impl SliderBuilder {
    pub fn new() -> Self {
        let rect_color = [0.1, 0.1, 0.1, 1.0];
        let style = vec![RectStyleField::BackgroundColor(Value::Single(rect_color))];
        let mut widget = WidgetBuilder::new();
        widget.set_drawable_with_style(RectDrawable::new(), style);
        widget.layout().dimensions(Dimensions {
            width: 200.0,
            height: 30.0,
        });

        let rect_color = [0.4, 0.4, 0.4, 1.0];
        let style = vec![RectStyleField::BackgroundColor(Value::Single(rect_color))];
        let mut slider_handle = WidgetBuilder::new();
        slider_handle
            .set_drawable_with_style(RectDrawable::new(), style)
            .add_handler(DragHandler::new(widget.id()))
            .make_draggable();
        slider_handle.layout().dimensions(Dimensions {
            width: 30.0,
            height: 30.0,
        });

        widget.add_child(slider_handle);
        SliderBuilder { widget: widget }
    }
    pub fn on_val_changed<F>(&mut self, on_val_changed: F) -> &mut Self
        where F: Fn(f64) + 'static
    {
        self.widget.add_handler(SliderHandler::new(on_val_changed));
        self
    }
}

pub struct SliderHandler<F: Fn(f64)> {
    callback: F,
}
impl<F: Fn(f64)> SliderHandler<F> {
    pub fn new(callback: F) -> Self {
        SliderHandler { callback: callback }
    }
}
impl<F> WidgetEventHandler<MovedSliderWidgetEvent> for SliderHandler<F>
    where F: Fn(f64) {
    fn handle(&mut self, event: &MovedSliderWidgetEvent, mut args: WidgetEventArgs) {
        let bounds = args.widget.layout.bounds();
        let range = bounds.width - (event.slider_right - event.slider_left);
        let val = (event.slider_left - bounds.left) / range;
        (self.callback)(val);
        args.event_state.handled = true;
    }
}

struct MovedSliderWidgetEvent {
    slider_left: f64,
    slider_right: f64,
}
struct DragHandler {
    container: WidgetId,
    start_pos: f64,
}
impl DragHandler {
    pub fn new(container: WidgetId) -> Self {
        DragHandler { container: container, start_pos: 0.0 }
    }
}
impl WidgetEventHandler<WidgetDrag> for DragHandler {
    fn handle(&mut self, event: &WidgetDrag, args: WidgetEventArgs) {
        let WidgetEventArgs { solver, widget, .. } = args;
        let ref layout = widget.layout;
        let bounds = layout.bounds();
        let &WidgetDrag { ref drag_type, position } = event;
        let drag_pos = position.x;
        match *drag_type {
            DragEvent::DragStart => {
                self.start_pos = drag_pos - bounds.left;
            }
            _ => {
                solver.update_solver(|solver| {
                    if !solver.has_edit_variable(&layout.left) {
                        solver.add_edit_variable(layout.left, STRONG).unwrap();
                    }
                    solver.suggest_value(layout.left, drag_pos - self.start_pos).unwrap();
                });
                let event = MovedSliderWidgetEvent { slider_left: bounds.left, slider_right: bounds.right() };
                args.queue.push(Target::Widget(self.container), event);
            }
        }
    }
}
