//! The shadow tree, used the way a host and an author would: build, read, edit, lay out,
//! commit.

use std::any::Any;
use std::sync::Arc;

use folio::{
    Color, ImageProps, ImageShadowNode, ImageState, Inline, LayoutSystems, LayoutableShadowNode, Revision, ShadowNode,
    ShadowNodeKind, ShadowTree, Stale, TextAttributes, TextItem, TextProps, TextShadowNode, TextSpanProps,
    TextSpanShadowNode, TextStyle, ViewProps, ViewShadowNode, WindowShadowNode, WindowState, edit,
};
use hooke::{LayoutCx, RootConstraints, Scratch, Size};

use elsewhere::{SwitchProps, SwitchShadowNode};

/// A kind the enum does not list, written as another crate would write it.
mod elsewhere {
    use std::sync::Arc;

    use folio::{Family, LayoutableShadowNode, Props, ShadowNode, ShadowNodeKind, ViewProps};
    use hooke::{Children, LayoutState, Parts, PartsMut};

    #[derive(Clone, Debug, Default)]
    pub struct SwitchProps {
        pub view: ViewProps,
        pub on: bool,
    }

    impl Props for SwitchProps {}

    #[derive(Clone, Debug)]
    pub struct SwitchShadowNode {
        family: Arc<Family>,
        props: Arc<SwitchProps>,
        children: Children<ShadowNodeKind>,
        layout: LayoutState,
    }

    impl SwitchShadowNode {
        pub fn new(props: SwitchProps) -> Self {
            SwitchShadowNode {
                family: Family::new(),
                props: Arc::new(props),
                children: Children::empty(),
                layout: LayoutState::measured(0),
            }
        }

        pub fn is_on(&self) -> bool {
            self.props.on
        }
    }

    impl ShadowNode for SwitchShadowNode {
        fn family(&self) -> &Arc<Family> {
            &self.family
        }

        fn props(&self) -> &dyn Props {
            &*self.props
        }
    }

    impl LayoutableShadowNode for SwitchShadowNode {
        fn view_props(&self) -> &ViewProps {
            &self.props.view
        }

        fn layout_parts(&self) -> Parts<'_, ShadowNodeKind> {
            Parts { style: &self.props.view.layout, state: &self.layout, children: &self.children }
        }

        fn layout_parts_mut(&mut self) -> PartsMut<'_, ShadowNodeKind> {
            PartsMut { style: &self.props.view.layout, state: &mut self.layout, children: &mut self.children }
        }
    }
}

/// The application: a root view holding a main window, whose content is a card holding a
/// label, a picture, a switch and a second window.
struct App {
    tree: ShadowTree,
    /// The revision in which the main window was added to the root.
    first: Revision,
    main: Arc<ShadowNodeKind>,
    card: Arc<ShadowNodeKind>,
    label: Arc<ShadowNodeKind>,
    emoji: Arc<ShadowNodeKind>,
    picture: Arc<ShadowNodeKind>,
    switch: Arc<ShadowNodeKind>,
    popup: Arc<ShadowNodeKind>,
    tooltip: Arc<ShadowNodeKind>,
}

fn node(kind: impl Into<ShadowNodeKind>) -> Arc<ShadowNodeKind> {
    Arc::new(kind.into())
}

fn build() -> App {
    let tree = ShadowTree::new();

    // The one view among the label's inline children is listed as an attachment.
    let emoji = node(ImageShadowNode::new(ImageProps::default()));
    let bold = TextSpanShadowNode::new(
        TextSpanProps { style: Arc::new(TextStyle { font_weight: Some(700), ..Default::default() }) },
        [Inline::Run("world".into()), Inline::Attachment(0)],
    );
    let label = node(TextShadowNode::new(
        TextProps::default(),
        [Inline::Run("Hello, ".into()), Inline::Span(Arc::new(bold))],
        [emoji.clone()],
    ));
    let picture = node(ImageShadowNode::new(ImageProps { source: Some("avatar.png".into()), ..Default::default() }));
    let switch =
        Arc::new(ShadowNodeKind::Other(Box::new(SwitchShadowNode::new(SwitchProps { on: true, ..Default::default() }))));

    // The second window is declared inside the content of the first.
    let tooltip = node(ViewShadowNode::new(ViewProps::default(), []));
    let popup = node(WindowShadowNode::new(ViewProps::default(), [tooltip.clone()]));
    let card = node(ViewShadowNode::new(
        ViewProps::default(),
        [label.clone(), picture.clone(), switch.clone(), popup.clone()],
    ));
    let main = node(WindowShadowNode::new(ViewProps::default(), [card.clone()]));

    let base = tree.current();
    let mut root = base.root.clone();
    let ShadowNodeKind::View(application) = Arc::make_mut(&mut root) else { panic!("the root is a view") };
    application.set_children([main.clone()]);
    let first = tree.try_commit(&base, root).unwrap();

    App { tree, first, main, card, label, emoji, picture, switch, popup, tooltip }
}

fn child(node: &ShadowNodeKind, index: usize) -> &Arc<ShadowNodeKind> {
    &node.layout_parts().children[index].node
}

fn size(node: &ShadowNodeKind) -> Size<f32> {
    node.layout_parts().state.size()
}

fn lay_out(root: &mut Arc<ShadowNodeKind>, scratch: &mut Scratch<f32>) -> Size<f32> {
    hooke::layout(root, RootConstraints::viewport(0.0, 0.0), &mut LayoutCx::new(scratch), &mut LayoutSystems).unwrap()
}

/// The host opens both windows, and the tree is laid out and committed.
fn open_windows(app: &App, scratch: &mut Scratch<f32>) -> (Revision, Size<f32>) {
    let mut root = app.first.root.clone();
    for (window, state) in
        [(&app.main, WindowState::sized(390.0, 844.0, 3.0)), (&app.popup, WindowState::sized(200.0, 100.0, 2.0))]
    {
        let found = edit(&mut root, window.family(), |node| {
            let ShadowNodeKind::Window(window) = node else { panic!("not a window") };
            window.set_state(state);
        });
        assert!(found);
    }
    let root_size = lay_out(&mut root, scratch);
    (app.tree.try_commit(&app.first, root).unwrap(), root_size)
}

/// The host reports the picture's size, and the tree is laid out again and committed.
fn report_picture(app: &App, base: &Revision, scratch: &mut Scratch<f32>) -> Revision {
    let mut root = base.root.clone();
    let found = edit(&mut root, app.picture.family(), |node| {
        let ShadowNodeKind::Image(image) = node else { panic!("not an image") };
        image.set_state(ImageState { natural_size: Some(Size::new(64.0, 64.0)) });
    });
    assert!(found);
    lay_out(&mut root, scratch);
    app.tree.try_commit(base, root).unwrap()
}

#[test]
fn building_places_the_main_window_under_the_root() {
    let app = build();
    assert_eq!(app.first.number, 1);
    assert!(Arc::ptr_eq(child(&app.first.root, 0), &app.main));
    assert!(Arc::ptr_eq(child(&app.main, 0), &app.card));
    assert!(Arc::ptr_eq(child(&app.popup, 0), &app.tooltip));
}

#[test]
fn reading_through_the_traits() {
    let app = build();
    assert_eq!(app.card.layout_parts().children.len(), 4);
    assert_eq!(app.switch.view_props().appearance.opacity, 1.0);
    assert!(Arc::ptr_eq(&app.label.family().parent().unwrap(), app.card.family()));
    assert!(Arc::ptr_eq(&app.card.family().parent().unwrap(), app.main.family()));
}

#[test]
fn reading_by_match_on_the_kind() {
    let app = build();
    let ShadowNodeKind::Text(text) = &*app.label else { panic!("the label is text") };
    assert_eq!(text.content().len(), 2);
    let ShadowNodeKind::Image(picture) = &*app.picture else { panic!("the picture is an image") };
    assert_eq!(picture.props().source.as_deref(), Some("avatar.png"));
    assert!(picture.state().is_none());
}

#[test]
fn downcasting_a_custom_kind() {
    let app = build();
    let ShadowNodeKind::Other(other) = &*app.switch else { panic!("the switch is another kind") };
    let other: &dyn Any = &**other;
    assert!(other.downcast_ref::<SwitchShadowNode>().unwrap().is_on());
}

#[test]
fn text_items_apply_the_style_cascade() {
    let app = build();
    let ShadowNodeKind::Text(text) = &*app.label else { panic!("the label is text") };
    let inherited = TextAttributes {
        color: Color(0x000000ff),
        font_family: "system".into(),
        font_size: 17.0,
        font_weight: 400,
        italic: false,
        letter_spacing: 0.0,
    };
    let items = text.items(&inherited);
    assert_eq!(items.len(), 3);
    assert!(matches!(&items[0], TextItem::Characters(characters, attributes)
        if &**characters == "Hello, " && attributes.font_weight == 400));
    assert!(matches!(&items[1], TextItem::Characters(characters, attributes)
        if &**characters == "world" && attributes.font_weight == 700));
    assert_eq!(items[2], TextItem::Attachment(0));
}

#[test]
fn one_layout_pass_lays_out_both_windows() {
    let app = build();
    let mut scratch = Scratch::new();
    let (second, root_size) = open_windows(&app, &mut scratch);

    let main_window = child(&second.root, 0);
    let card = child(main_window, 0);
    let popup_window = child(card, 3);
    assert_eq!(root_size, Size::new(0.0, 0.0));
    assert_eq!(size(main_window), Size::new(0.0, 0.0));
    assert_eq!(size(card), Size::new(390.0, 844.0));
    assert_eq!(size(popup_window), Size::new(0.0, 0.0));
    assert_eq!(size(child(popup_window, 0)), Size::new(200.0, 100.0));
}

#[test]
fn an_edit_copies_only_the_path_to_the_target() {
    let app = build();
    let mut scratch = Scratch::new();
    let (second, _) = open_windows(&app, &mut scratch);
    let third = report_picture(&app, &second, &mut scratch);

    let old_card = child(child(&second.root, 0), 0);
    let new_card = child(child(&third.root, 0), 0);
    assert!(!Arc::ptr_eq(old_card, new_card));
    assert!(Arc::ptr_eq(old_card.family(), new_card.family()));
    assert!(Arc::ptr_eq(child(old_card, 0), child(new_card, 0)), "the label is shared");
    assert!(Arc::ptr_eq(child(old_card, 3), child(new_card, 3)), "and so is the second window");
    assert_eq!(size(child(new_card, 1)).width, 64.0);
}

#[test]
fn edit_finds_a_view_inside_a_nested_window() {
    let app = build();
    let mut root = app.first.root.clone();
    let mut reached = None;
    assert!(edit(&mut root, app.tooltip.family(), |node| reached = Some(node.family().id())));
    assert_eq!(reached, Some(app.tooltip.family().id()));
}

#[test]
fn edit_finds_an_attachment_inside_text() {
    let app = build();
    let mut root = app.first.root.clone();
    let mut reached = None;
    assert!(edit(&mut root, app.emoji.family(), |node| reached = Some(node.family().id())));
    assert_eq!(reached, Some(app.emoji.family().id()));
}

#[test]
fn edit_returns_false_for_a_family_not_in_the_tree() {
    let app = build();
    let stranger = node(ViewShadowNode::new(ViewProps::default(), []));
    let mut root = app.first.root.clone();
    let mut called = false;
    assert!(!edit(&mut root, stranger.family(), |_| called = true));
    assert!(!called);
    assert!(Arc::ptr_eq(&root, &app.first.root));
}

#[test]
fn a_stale_commit_is_refused_with_the_current_revision() {
    let app = build();
    let mut scratch = Scratch::new();
    let (second, _) = open_windows(&app, &mut scratch);
    report_picture(&app, &second, &mut scratch);

    let refused = app.tree.try_commit(&app.first, app.first.root.clone());
    assert!(matches!(refused, Err(Stale(now)) if now.number == 3));
}

#[test]
#[should_panic(expected = "a family never changes its parent")]
fn a_family_never_changes_its_parent() {
    let leaf = node(ViewShadowNode::new(ViewProps::default(), []));
    let _first = ViewShadowNode::new(ViewProps::default(), [leaf.clone()]);
    let _second = ViewShadowNode::new(ViewProps::default(), [leaf]);
}
