pub mod graph;
pub mod queue;
pub mod layout;

pub use self::graph::WidgetGraph;
pub use self::queue::{EventQueue, EventAddress};
pub use self::layout::LimnSolver;

use input::mouse::{MouseMoveHandler, MouseButtonHandler, MouseWheelHandler, MouseLayoutChangeHandler, MouseController};
use input::mouse::{MouseMoved, MouseButton, MouseWheel};
use input::keyboard::{FocusHandler, KeyboardForwarder};
use input::keyboard::{KeyboardInput};

use backend::Window;

use std::any::{Any, TypeId};

use glutin;

use util::Point;
use resources::WidgetId;

use widget::WidgetBuilder;

pub struct Ui {
    pub graph: WidgetGraph,
    pub solver: LimnSolver,
}

impl Ui {
    pub fn new(window: &mut Window, event_queue: &EventQueue) -> Self {
        let graph = WidgetGraph::new(window);
        let solver = LimnSolver::new(event_queue.clone());
        Ui {
            graph: graph,
            solver: solver,
        }
    }

    pub fn set_root(&mut self, root_widget: WidgetBuilder, window: &mut Window) {
        self.graph.set_root(root_widget, &mut self.solver);
        self.graph.resize_window_to_fit(&window, &mut self.solver);
    }

    pub fn handle_input(&mut self, event: glutin::Event, event_queue: &mut EventQueue) {
        let ref root_widget = self.graph.get_root();
        let all_widgets = EventAddress::SubTree(root_widget.id);
        match event {
            glutin::Event::MouseWheel(mouse_scroll_delta, _) => {
                event_queue.push(all_widgets, MouseWheel(mouse_scroll_delta));
                event_queue.push(EventAddress::Ui, MouseWheel(mouse_scroll_delta));
            }
            glutin::Event::MouseInput(state, button) => {
                event_queue.push(all_widgets, MouseButton(state, button));
                event_queue.push(EventAddress::Ui, MouseButton(state, button));
            }
            glutin::Event::MouseMoved(x, y) => {
                let point = Point::new(x as f64, y as f64);
                event_queue.push(all_widgets, MouseMoved(point));
                event_queue.push(EventAddress::Ui, MouseMoved(point));
            }
            glutin::Event::KeyboardInput(state, scan_code, maybe_keycode) => {
                let key_input = KeyboardInput(state, scan_code, maybe_keycode);
                event_queue.push(EventAddress::Ui, key_input);
            }
            _ => (),
        }
    }

    pub fn layout_changed(&mut self, event: &LayoutChanged, event_queue: &mut EventQueue) {
        let &LayoutChanged(widget_id) = event;
        if let Some(widget) = self.graph.get_widget(widget_id) {
            widget.layout.update(&mut self.solver);
        }
        // redraw everything when layout changes, for now
        event_queue.push(EventAddress::Ui, RedrawEvent);
        self.graph.redraw();
    }
}

pub struct EventArgs<'a> {
    pub ui: &'a mut Ui,
    pub event_queue: &'a mut EventQueue,
}

pub trait EventHandler<T> {
    fn handle(&mut self, event: &T, args: EventArgs);
}

pub struct HandlerWrapper {
    type_id: TypeId,
    handler: Box<Any>,
    handle_fn: Box<Fn(&mut Box<Any>, &Box<Any + Send>, EventArgs)>,
}
impl HandlerWrapper {
    pub fn new<H, E>(handler: H) -> Self
        where H: EventHandler<E> + 'static,
              E: 'static
    {
        let handle_fn = |handler: &mut Box<Any>, event: &Box<Any + Send>, args: EventArgs| {
            let event: &E = event.downcast_ref().unwrap();
            let handler: &mut H = handler.downcast_mut().unwrap();
            handler.handle(event, args);
        };
        HandlerWrapper {
            type_id: TypeId::of::<E>(),
            handler: Box::new(handler),
            handle_fn: Box::new(handle_fn),
        }
    }
    pub fn handles(&self, type_id: TypeId) -> bool {
        self.type_id == type_id
    }
    pub fn handle(&mut self, event: &Box<Any + Send>, args: EventArgs) {
        (self.handle_fn)(&mut self.handler, event, args);
    }
}


pub struct InputEvent(pub glutin::Event);
pub struct RedrawEvent;
pub struct LayoutChanged(pub WidgetId);

pub struct InputHandler;
impl EventHandler<InputEvent> for InputHandler {
    fn handle(&mut self, event: &InputEvent, mut args: EventArgs) {
        args.ui.handle_input(event.0.clone(), &mut args.event_queue);
    }
}

pub struct RedrawHandler;
impl EventHandler<RedrawEvent> for RedrawHandler {
    fn handle(&mut self, _: &RedrawEvent, args: EventArgs) {
        args.ui.graph.redraw();
    }
}
pub struct LayoutChangeHandler;
impl EventHandler<LayoutChanged> for LayoutChangeHandler {
    fn handle(&mut self, event: &LayoutChanged, args: EventArgs) {
        args.ui.layout_changed(event, args.event_queue);
    }
}
pub fn get_default_event_handlers() -> Vec<HandlerWrapper> {
    vec![
        HandlerWrapper::new(RedrawHandler),
        HandlerWrapper::new(LayoutChangeHandler),
        HandlerWrapper::new(InputHandler),
        HandlerWrapper::new(MouseController::new()),
        HandlerWrapper::new(MouseLayoutChangeHandler),
        HandlerWrapper::new(MouseMoveHandler),
        HandlerWrapper::new(MouseButtonHandler),
        HandlerWrapper::new(MouseWheelHandler),
        HandlerWrapper::new(KeyboardForwarder),
        HandlerWrapper::new(FocusHandler::new()),
    ]
}