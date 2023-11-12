use crate::{graphics::Rect, TabId, View, ViewId};
use slotmap::{HopSlotMap, SparseSecondaryMap};

// TODO(theofabilous): put this in some more global module?
// its likely useful in other places
// NOTE: This is a workaround for, say:
//    fn foo<'a>(arg: &'a T) -> impl MyTrait + 'a
// which actually means that the return type *outlives* 'a
// XXX: See: https://www.youtube.com/watch?v=CWiz_RtA1Hw&t=1527s
// XXX: This will apparently change in a future version of rust
pub trait Captures<U> {}
impl<T: ?Sized, U> Captures<U> for T {}

#[derive(Debug)]
pub struct Tabs {
    trees: HopSlotMap<TabId, Tree>,
    nodes: HopSlotMap<ViewId, Node>,
    pub focus: TabId,
}

// the dimensions are recomputed on window resize/tree change.
//
#[derive(Debug)]
pub struct Tree {
    pub(self) id: TabId,
    pub(self) root: ViewId,
    // (container, index inside the container)
    pub focus: ViewId,
    // fullscreen: bool,
    pub(self) area: Rect,

    pub(self) nodes: SparseSecondaryMap<ViewId, ()>,

    // used for traversals
    pub(self) stack: Vec<(ViewId, Rect)>,
}

pub struct TabProxy<'a> {
    pub(self) tabs: &'a Tabs,
    pub(self) tab: TabId,
}

pub struct TabProxyMut<'a> {
    pub(self) tabs: &'a mut Tabs,
    pub(self) tab: TabId,
}

#[derive(Debug)]
pub struct Node {
    parent: ViewId,
    content: Content,
}

#[derive(Debug)]
pub enum Content {
    View(Box<View>),
    Container(Box<Container>),
}

impl Node {
    pub fn container(layout: Layout) -> Self {
        Self {
            parent: ViewId::default(),
            content: Content::Container(Box::new(Container::new(layout))),
        }
    }

    pub fn view(view: View) -> Self {
        Self {
            parent: ViewId::default(),
            content: Content::View(Box::new(view)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical,
    // could explore stacked/tabbed
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Container {
    layout: Layout,
    children: Vec<ViewId>,
    area: Rect,
}

impl Container {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            children: Vec::new(),
            area: Rect::default(),
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new(Layout::Vertical)
    }
}

pub trait Tab {
    fn tab_id(&self) -> TabId;
    fn tabs(&self) -> &Tabs;

    #[inline(always)]
    fn tree(&self) -> &Tree {
        self.tabs().get_tree(self.tab_id())
    }

    #[inline(always)]
    fn focused(&self) -> ViewId {
        self.tree().focus
    }

    #[inline(always)]
    fn get_focused(&self) -> &View {
        self.get(self.focused())
    }

    #[inline(always)]
    fn get(&self, index: ViewId) -> &View {
        self.try_get(index).unwrap()
    }

    #[inline(always)]
    fn try_get(&self, index: ViewId) -> Option<&View> {
        self.tabs().try_get(index)
    }

    #[inline(always)]
    fn contains(&self, index: ViewId) -> bool {
        self.tabs().tab_contains(self.tab_id(), index).unwrap()
    }

    #[inline(always)]
    fn is_empty(&self) -> bool {
        self.tabs().tab_is_empty(self.tab_id()).unwrap()
    }

    #[inline(always)]
    fn find_split_in_direction(&self, id: ViewId, direction: Direction) -> Option<ViewId> {
        self.tabs()
            .find_split_in_direction(self.tab_id(), id, direction)
    }

    #[inline(always)]
    fn prev(&self) -> ViewId {
        self.tabs().prev(self.tab_id())
    }

    #[inline(always)]
    fn next(&self) -> ViewId {
        self.tabs().next(self.tab_id())
    }
}

pub trait TabMut: Tab {
    fn tabs_mut(&mut self) -> &mut Tabs;

    #[inline(always)]
    fn tree_mut<'a>(&'a mut self) -> &'a mut Tree {
        let tab_id = self.tab_id();
        self.tabs_mut().get_tree_mut(tab_id)
    }

    #[inline(always)]
    fn set_focused(&mut self, index: ViewId) {
        self.tree_mut().focus = index;
    }

    #[inline(always)]
    fn insert(&mut self, view: View) -> ViewId {
        let tab_id = self.tab_id();
        self.tabs_mut().insert(tab_id, view)
    }

    #[inline(always)]
    fn split(&mut self, view: View, layout: Layout) -> ViewId {
        let tab_id = self.tab_id();
        self.tabs_mut().split(tab_id, view, layout)
    }

    #[inline(always)]
    fn remove(&mut self, index: ViewId) {
        let tab_id = self.tab_id();
        self.tabs_mut().remove(tab_id, index)
    }

    #[inline(always)]
    fn get_mut(&mut self, index: ViewId) -> &mut View {
        self.tabs_mut().get_mut(index)
    }

    #[inline(always)]
    fn resize(&mut self, area: Rect) -> bool {
        let tab_id = self.tab_id();
        self.tabs_mut().resize_tab(tab_id, area)
    }

    #[inline(always)]
    fn recalculate(&mut self) {
        let tab_id = self.tab_id();
        self.tabs_mut().recalculate_tab(tab_id)
    }

    #[inline(always)]
    fn transpose(&mut self) {
        let tab_id = self.tab_id();
        self.tabs_mut().transpose(tab_id)
    }

    #[inline(always)]
    fn swap_split_in_direction(&mut self, direction: Direction) -> Option<()> {
        let tab_id = self.tab_id();
        self.tabs_mut().swap_split_in_direction(tab_id, direction)
    }
}

impl<'a> Tab for TabProxy<'a> {
    #[inline(always)]
    fn tab_id(&self) -> TabId {
        self.tab
    }

    #[inline(always)]
    fn tabs(&self) -> &Tabs {
        self.tabs
    }
}

impl<'a> Tab for TabProxyMut<'a> {
    #[inline(always)]
    fn tab_id(&self) -> TabId {
        self.tab
    }

    #[inline(always)]
    fn tabs(&self) -> &Tabs {
        self.tabs
    }
}

impl<'a> TabMut for TabProxyMut<'a> {
    #[inline(always)]
    fn tabs_mut(&mut self) -> &mut Tabs {
        self.tabs
    }
}

// impl trait return types can't be used in traits..
impl<'a> TabProxy<'a> {
    #[inline(always)]
    pub fn views(&self) -> impl Iterator<Item = (&View, bool)> {
        self.tabs().tab_views(self.tab_id())
    }
}

impl<'a> TabProxyMut<'a> {
    #[inline(always)]
    pub fn views_mut(&mut self) -> impl Iterator<Item = (&mut View, bool)> {
        let tab_id = self.tab_id();
        self.tabs_mut().tab_views_mut(tab_id)
    }
}

impl Tabs {
    pub fn new(area: Rect) -> Self {
        let nodes = HopSlotMap::with_key();
        let trees = HopSlotMap::with_key();
        let mut this = Self {
            focus: TabId::default(),
            trees,
            nodes,
        };
        this.focus = this.new_tree(area);
        this
    }

    fn new_tree(&mut self, area: Rect) -> TabId {
        let root = Node::container(Layout::Vertical);
        let root = self.nodes.insert(root);
        self.nodes[root].parent = root;

        let mut tree = Tree {
            id: TabId::default(),
            root,
            focus: root,
            area,
            nodes: SparseSecondaryMap::new(),
            stack: Vec::new(),
        };
        tree.nodes.insert(root, ());

        let tab_id = self.trees.insert(tree);
        self.trees.get_mut(tab_id).unwrap().id = tab_id;
        tab_id
    }

    pub fn new_tab(&mut self) -> TabId {
        let area = self.area(self.focus);
        let tab_id = self.new_tree(area);
        self.focus = tab_id;
        tab_id
    }

    pub fn close_tab(&mut self, tab: TabId) -> Option<TabId> {
        if self.len() == 1 {
            return None;
        }
        let tab = self.trees.remove(tab).unwrap();
        for index in tab.nodes.keys() {
            self.nodes.remove(index);
        }
        if self.focus != tab.id {
            Some(self.focus)
        } else {
            Some(self.focus_previous())
        }
    }

    pub fn iter_view_ids<'a>(
        &'a self,
        tab: TabId,
    ) -> impl Iterator<Item = ViewId> + Captures<&'a ()> {
        self.get_tree(tab)
            .nodes
            .keys()
            .filter(|id| match self.nodes.get(*id) {
                Some(Node {
                    content: Content::View(_),
                    ..
                }) => true,
                _ => false,
            })
    }

    #[inline]
    pub fn view_ids(&self, tab: TabId) -> Vec<ViewId> {
        self.iter_view_ids(tab).collect()
    }

    pub fn focus_next(&mut self) -> TabId {
        let curr = self.focus;
        let mut iter = self.trees.keys().skip_while(|tab| *tab != curr);
        iter.next();
        let id = iter.next().or_else(|| self.trees.keys().next()).unwrap();
        self.focus = id;
        id
    }

    pub fn focus_previous(&mut self) -> TabId {
        let curr = self.focus;
        let iter = self.trees.keys().take_while(|tab| *tab != curr);
        let id = iter.last().or_else(|| self.trees.keys().last()).unwrap();
        self.focus = id;
        id
    }

    #[inline]
    pub fn iter_tabs(&self) -> impl Iterator<Item = (TabId, &Tree)> {
        self.trees.iter()
    }

    #[inline]
    pub fn try_get_tree(&self, tab_id: TabId) -> Option<&Tree> {
        self.trees.get(tab_id)
    }

    #[inline]
    pub fn get_tree(&self, tab_id: TabId) -> &Tree {
        self.try_get_tree(tab_id).unwrap()
    }

    #[inline]
    pub fn try_get_tree_mut(&mut self, tab_id: TabId) -> Option<&mut Tree> {
        self.trees.get_mut(tab_id)
    }

    #[inline]
    pub fn get_tree_mut(&mut self, tab_id: TabId) -> &mut Tree {
        self.try_get_tree_mut(tab_id).unwrap()
    }

    #[inline]
    pub fn try_get_focused_view_for_tab(&self, tab: TabId) -> Option<ViewId> {
        Some(self.try_get_tree(tab)?.focus)
    }

    #[inline]
    pub fn get_focused_view_for_tab(&self, tab: TabId) -> ViewId {
        self.get_tree(tab).focus
    }

    #[inline]
    pub fn focused_view(&self) -> ViewId {
        self.get_tree(self.focus).focus
    }

    #[inline]
    pub fn set_focused_view(&mut self, tab: TabId, index: ViewId) {
        self.get_tree_mut(tab).focus = index;
    }

    #[inline]
    pub fn set_focused_view_for_current_tab(&mut self, index: ViewId) {
        self.set_focused_view(self.focus, index)
    }

    #[inline]
    pub fn curr_tree(&self) -> TabProxy {
        let focus = self.focus;
        TabProxy {
            tabs: self,
            tab: focus,
        }
    }

    #[inline]
    pub fn curr_tree_mut(&mut self) -> TabProxyMut {
        let focus = self.focus;
        TabProxyMut {
            tabs: self,
            tab: focus,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.trees.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    pub fn insert(&mut self, tab: TabId, view: View) -> ViewId {
        let focus = self.get_tree_mut(tab).focus;
        let parent = self.nodes[focus].parent;
        let mut node = Node::view(view);
        node.parent = parent;
        let node = self.nodes.insert(node);
        self.get_mut(node).id = node;

        let container = match &mut self.nodes[parent] {
            Node {
                content: Content::Container(container),
                ..
            } => container,
            _ => unreachable!(),
        };

        // insert node after the current item if there is children already
        let pos = if container.children.is_empty() {
            0
        } else {
            let pos = container
                .children
                .iter()
                .position(|&child| child == focus)
                .unwrap();
            pos + 1
        };

        container.children.insert(pos, node);
        // focus the new node
        let mut tree = self.get_tree_mut(tab);
        tree.focus = node;
        tree.nodes.insert(node, ());

        // recalculate all the sizes
        self.recalculate();

        node
    }

    pub fn split(&mut self, tab: TabId, view: View, layout: Layout) -> ViewId {
        let focus = self.get_tree_mut(tab).focus;
        let parent = self.nodes[focus].parent;

        let node = Node::view(view);
        let node = self.nodes.insert(node);
        self.get_mut(node).id = node;

        let container = match &mut self.nodes[parent] {
            Node {
                content: Content::Container(container),
                ..
            } => container,
            _ => unreachable!(),
        };
        if container.layout == layout {
            // insert node after the current item if there is children already
            let pos = if container.children.is_empty() {
                0
            } else {
                let pos = container
                    .children
                    .iter()
                    .position(|&child| child == focus)
                    .unwrap();
                pos + 1
            };
            container.children.insert(pos, node);
            self.nodes[node].parent = parent;
        } else {
            let mut split = Node::container(layout);
            split.parent = parent;
            let split = self.nodes.insert(split);

            let container = match &mut self.nodes[split] {
                Node {
                    content: Content::Container(container),
                    ..
                } => container,
                _ => unreachable!(),
            };
            container.children.push(focus);
            container.children.push(node);
            self.nodes[focus].parent = split;
            self.nodes[node].parent = split;

            let container = match &mut self.nodes[parent] {
                Node {
                    content: Content::Container(container),
                    ..
                } => container,
                _ => unreachable!(),
            };

            let pos = container
                .children
                .iter()
                .position(|&child| child == focus)
                .unwrap();

            // replace focus on parent with split
            container.children[pos] = split;
        }

        // focus the new node
        let mut tree = self.get_tree_mut(tab);
        tree.focus = node;
        tree.nodes.insert(node, ());

        // recalculate all the sizes
        self.recalculate();

        node
    }

    pub fn remove(&mut self, tab: TabId, index: ViewId) {
        let mut stack = Vec::new();
        {
            let tree = self.get_tree(tab);

            // XXX
            // TODO(theofabilous): handle focus next tab?
            // XXX
            if self.focus == tree.id && tree.focus == index {
                let prev_view = self.prev(tab);
                let mut tree = self.get_tree_mut(tab);
                // focus on something else
                tree.focus = prev_view;
            }
        }

        stack.push(index);

        while let Some(index) = stack.pop() {
            let parent_id = self.nodes[index].parent;
            if let Node {
                content: Content::Container(container),
                ..
            } = &mut self.nodes[parent_id]
            {
                if let Some(pos) = container.children.iter().position(|&child| child == index) {
                    container.children.remove(pos);
                    // TODO: if container now only has one child, remove it and place child in parent
                    if container.children.is_empty() && parent_id != self.get_tree(tab).root {
                        // if container now empty, remove it
                        stack.push(parent_id);
                    }
                }
            }
            self.get_tree_mut(tab).nodes.remove(index);
            self.nodes.remove(index);
        }

        self.recalculate()
    }

    pub fn tab_views<'a>(&'a self, tab: TabId) -> impl Iterator<Item = (&'a View, bool)> {
        let tree = self.get_tree(tab);
        let focus = tree.focus;
        tree.nodes
            .keys()
            .filter_map(move |key| match self.nodes.get(key).unwrap() {
                Node {
                    content: Content::View(view),
                    ..
                } => Some((view.as_ref(), focus == key)),
                _ => None,
            })
    }

    pub fn tab_views_mut(&mut self, tab_id: TabId) -> impl Iterator<Item = (&mut View, bool)> {
        let tree = self.trees.get(tab_id).unwrap();
        let tree_nodes = &tree.nodes;
        let focus = tree.focus;
        self.nodes.iter_mut().filter_map(move |(key, node)| {
            if let None = tree_nodes.get(key) {
                None
            } else {
                match node {
                    Node {
                        content: Content::View(view),
                        ..
                    } => Some((view.as_mut(), focus == key)),
                    _ => None,
                }
            }
        })
    }

    pub fn all_views(&self) -> impl Iterator<Item = (&View, bool)> {
        let focus = self.trees.get(self.focus).unwrap().focus;
        self.nodes.iter().filter_map(move |(key, node)| match node {
            Node {
                content: Content::View(view),
                ..
            } => Some((view.as_ref(), focus == key)),
            _ => None,
        })
    }

    pub fn all_views_mut(&mut self) -> impl Iterator<Item = (&mut View, bool)> {
        let focus = self.trees.get(self.focus).unwrap().focus;
        self.nodes
            .iter_mut()
            .filter_map(move |(key, node)| match node {
                Node {
                    content: Content::View(view),
                    ..
                } => Some((view.as_mut(), focus == key)),
                _ => None,
            })
    }

    /// Get reference to a [View] by index.
    /// # Panics
    ///
    /// Panics if `index` is not in self.nodes, or if the node's content is not [Content::View]. This can be checked with [Self::contains].
    // pub fn get(&self, index: ViewId) -> &View {
    //     self.try_get(index).unwrap()
    // }
    pub fn get(&self, index: ViewId) -> &View {
        self.try_get(index).unwrap()
    }

    /// Try to get reference to a [View] by index. Returns `None` if node content is not a [`Content::View`].
    ///
    /// Does not panic if the view does not exists anymore.
    pub fn try_get(&self, index: ViewId) -> Option<&View> {
        match self.nodes.get(index) {
            Some(Node {
                content: Content::View(view),
                ..
            }) => Some(view),
            _ => None,
        }
    }

    /// Get a mutable reference to a [View] by index.
    /// # Panics
    ///
    /// Panics if `index` is not in self.nodes, or if the node's content is not [Content::View]. This can be checked with [Self::contains].
    pub fn get_mut(&mut self, index: ViewId) -> &mut View {
        match &mut self.nodes[index] {
            Node {
                content: Content::View(view),
                ..
            } => view,
            _ => unreachable!(),
        }
    }

    /// Check if any tree contains a [Node] with a given index.
    pub fn exists(&self, index: ViewId) -> bool {
        self.nodes.contains_key(index)
    }

    /// Check if tree contains a [Node] with a given index. Returns None if
    /// the tap does not exist.
    pub fn tab_contains(&self, tab_id: TabId, index: ViewId) -> Option<bool> {
        self.try_get_tree(tab_id)
            .and_then(move |tab| Some(tab.nodes.contains_key(index)))
    }

    pub fn tab_is_empty(&self, tab: TabId) -> Option<bool> {
        let tab = self.get_tree(tab);
        // TODO(theofabilous): this was giving 'invalid HopSlotMap' panic
        // when used without '.get()' when doing ":quit-all". Ensure that
        // it is normal.
        match &self.nodes.get(tab.root)? {
            Node {
                content: Content::Container(container),
                ..
            } => Some(container.children.is_empty()),
            _ => unreachable!(),
        }
    }

    pub fn all_empty(&self) -> bool {
        self.trees
            .keys()
            .all(|tab| self.tab_is_empty(tab).unwrap_or(true))
    }

    // TODO(theofabilous): what is area?
    // can it change from tree to tree?
    pub fn resize_tab(&mut self, tab: TabId, area: Rect) -> bool {
        let mut tree = self.get_tree_mut(tab);
        if tree.area != area {
            tree.area = area;
            self.recalculate_tab(tab);
            return true;
        }
        false
    }

    #[inline(always)]
    pub fn iter_tab_ids<'a>(&'a self) -> impl Iterator<Item = TabId> + Captures<&'a ()> {
        self.trees.keys()
    }

    #[inline]
    pub fn tab_ids(&self) -> Vec<TabId> {
        self.iter_tab_ids().collect()
    }

    pub fn resize(&mut self, area: Rect) -> bool {
        let mut result = false;
        for tab in self.tab_ids() {
            result |= self.resize_tab(tab, area);
        }
        result
    }

    // TODO(theofabilous): maybe this could be done lazily
    // i.e. only do it for the active tab, and recalc when
    // switching to a tab for which a recalc is needed
    pub fn recalculate(&mut self) {
        for tab in self.tab_ids() {
            self.recalculate_tab(tab)
        }
    }

    pub fn recalculate_tab(&mut self, tab: TabId) {
        match self.tab_is_empty(tab) {
            None => return,
            Some(true) => {
                let mut tree = self.get_tree_mut(tab);
                tree.focus = tree.root;
                return;
            }
            _ => (),
        }

        let tree = self.get_tree_mut(tab);
        let root = tree.root;
        let area = tree.area;
        let mut stack = std::mem::take(&mut tree.stack);

        stack.push((root, area));

        // take the area
        // fetch the node
        // a) node is view, give it whole area
        // b) node is container, calculate areas for each child and push them on the stack

        while let Some((key, area)) = stack.pop() {
            let node = &mut self.nodes[key];

            match &mut node.content {
                Content::View(view) => {
                    // debug!!("setting view area {:?}", area);
                    view.area = area;
                } // TODO: call f()
                Content::Container(container) => {
                    // debug!!("setting container area {:?}", area);
                    container.area = area;

                    match container.layout {
                        Layout::Horizontal => {
                            let len = container.children.len();

                            let height = area.height / len as u16;

                            let mut child_y = area.y;

                            for (i, child) in container.children.iter().enumerate() {
                                let mut area: Rect;
                                {
                                    area = Rect::new(
                                        container.area.x,
                                        child_y,
                                        container.area.width,
                                        height,
                                    );
                                }
                                child_y += height;

                                // last child takes the remaining width because we can get uneven
                                // space from rounding
                                if i == len - 1 {
                                    area.height = container.area.y + container.area.height - area.y;
                                }

                                stack.push((*child, area));
                            }
                        }
                        Layout::Vertical => {
                            let len = container.children.len();

                            let width = area.width / len as u16;

                            let inner_gap = 1u16;
                            // let total_gap = inner_gap * (len as u16 - 1);

                            let mut child_x = area.x;

                            for (i, child) in container.children.iter().enumerate() {
                                let mut area = Rect::new(
                                    child_x,
                                    container.area.y,
                                    width,
                                    container.area.height,
                                );
                                child_x += width + inner_gap;

                                // last child takes the remaining width because we can get uneven
                                // space from rounding
                                if i == len - 1 {
                                    area.width = container.area.x + container.area.width - area.x;
                                }

                                stack.push((*child, area));
                            }
                        }
                    }
                }
            }
        }

        self.get_tree_mut(tab).stack = stack;
    }

    #[inline]
    pub fn traverse(&self, tab: TabId) -> Traverse {
        Traverse::new(self, self.get_tree(tab))
    }

    // Finds the split in the given direction if it exists
    pub fn find_split_in_direction(
        &self,
        tab: TabId,
        id: ViewId,
        direction: Direction,
    ) -> Option<ViewId> {
        let parent = self.nodes[id].parent;
        // Base case, we found the root of the tree
        if parent == id {
            return None;
        }
        // Parent must always be a container
        let parent_container = match &self.nodes[parent].content {
            Content::Container(container) => container,
            Content::View(_) => unreachable!(),
        };

        match (direction, parent_container.layout) {
            (Direction::Up, Layout::Vertical)
            | (Direction::Left, Layout::Horizontal)
            | (Direction::Right, Layout::Horizontal)
            | (Direction::Down, Layout::Vertical) => {
                // The desired direction of movement is not possible within
                // the parent container so the search must continue closer to
                // the root of the split tree.
                self.find_split_in_direction(tab, parent, direction)
            }
            (Direction::Up, Layout::Horizontal)
            | (Direction::Down, Layout::Horizontal)
            | (Direction::Left, Layout::Vertical)
            | (Direction::Right, Layout::Vertical) => {
                // It's possible to move in the desired direction within
                // the parent container so an attempt is made to find the
                // correct child.
                match self.find_child(tab, id, &parent_container.children, direction) {
                    // Child is found, search is ended
                    Some(id) => Some(id),
                    // A child is not found. This could be because of either two scenarios
                    // 1. Its not possible to move in the desired direction, and search should end
                    // 2. A layout like the following with focus at X and desired direction Right
                    // | _ | x |   |
                    // | _ _ _ |   |
                    // | _ _ _ |   |
                    // The container containing X ends at X so no rightward movement is possible
                    // however there still exists another view/container to the right that hasn't
                    // been explored. Thus another search is done here in the parent container
                    // before concluding it's not possible to move in the desired direction.
                    None => self.find_split_in_direction(tab, parent, direction),
                }
            }
        }
    }

    fn find_child(
        &self,
        tab: TabId,
        id: ViewId,
        children: &[ViewId],
        direction: Direction,
    ) -> Option<ViewId> {
        let tree = self.try_get_tree(tab)?;
        let mut child_id = match direction {
            // index wise in the child list the Up and Left represents a -1
            // thus reversed iterator.
            Direction::Up | Direction::Left => children
                .iter()
                .rev()
                .skip_while(|i| **i != id)
                .copied()
                .nth(1)?,
            // Down and Right => +1 index wise in the child list
            Direction::Down | Direction::Right => {
                children.iter().skip_while(|i| **i != id).copied().nth(1)?
            }
        };
        let (current_x, current_y) = match &self.nodes[tree.focus].content {
            Content::View(current_view) => (current_view.area.left(), current_view.area.top()),
            Content::Container(_) => unreachable!(),
        };

        // If the child is a container the search finds the closest container child
        // visually based on screen location.
        while let Content::Container(container) = &self.nodes[child_id].content {
            match (direction, container.layout) {
                (_, Layout::Vertical) => {
                    // find closest split based on x because y is irrelevant
                    // in a vertical container (and already correct based on previous search)
                    child_id = *container.children.iter().min_by_key(|id| {
                        let x = match &self.nodes[**id].content {
                            Content::View(view) => view.area.left(),
                            Content::Container(container) => container.area.left(),
                        };
                        (current_x as i16 - x as i16).abs()
                    })?;
                }
                (_, Layout::Horizontal) => {
                    // find closest split based on y because x is irrelevant
                    // in a horizontal container (and already correct based on previous search)
                    child_id = *container.children.iter().min_by_key(|id| {
                        let y = match &self.nodes[**id].content {
                            Content::View(view) => view.area.top(),
                            Content::Container(container) => container.area.top(),
                        };
                        (current_y as i16 - y as i16).abs()
                    })?;
                }
            }
        }
        Some(child_id)
    }

    pub fn prev(&self, tab: TabId) -> ViewId {
        // This function is very dumb, but that's because we don't store any parent links.
        // (we'd be able to go parent.prev_sibling() recursively until we find something)
        // For now that's okay though, since it's unlikely you'll be able to open a large enough
        // number of splits to notice.

        let tree = self.get_tree(tab);
        let mut views = self
            .traverse(tab)
            .rev()
            .skip_while(|&(id, _view)| id != tree.focus)
            .skip(1); // Skip focused value
        if let Some((id, _)) = views.next() {
            id
        } else {
            // extremely crude, take the last item
            let (key, _) = self.traverse(tab).rev().next().unwrap();
            key
        }
    }

    pub fn next(&self, tab: TabId) -> ViewId {
        // This function is very dumb, but that's because we don't store any parent links.
        // (we'd be able to go parent.next_sibling() recursively until we find something)
        // For now that's okay though, since it's unlikely you'll be able to open a large enough
        // number of splits to notice.

        let tree = self.get_tree(tab);
        let mut views = self
            .traverse(tab)
            .skip_while(|&(id, _view)| id != tree.focus)
            .skip(1); // Skip focused value
        if let Some((id, _)) = views.next() {
            id
        } else {
            // extremely crude, take the first item again
            let (key, _) = self.traverse(tab).next().unwrap();
            key
        }
    }

    pub fn transpose(&mut self, tab: TabId) {
        let tree = self.get_tree(tab);
        let focus = tree.focus;
        let parent = self.nodes[focus].parent;
        if let Content::Container(container) = &mut self.nodes[parent].content {
            container.layout = match container.layout {
                Layout::Vertical => Layout::Horizontal,
                Layout::Horizontal => Layout::Vertical,
            };
            self.recalculate();
        }
    }

    pub fn swap_split_in_direction(&mut self, tab: TabId, direction: Direction) -> Option<()> {
        let tree = self.get_tree(tab);
        let focus = tree.focus;
        let target = self.find_split_in_direction(tab, focus, direction)?;
        let focus_parent = self.nodes[focus].parent;
        let target_parent = self.nodes[target].parent;

        if focus_parent == target_parent {
            let parent = focus_parent;
            let [parent, focus, target] = self.nodes.get_disjoint_mut([parent, focus, target])?;
            match (&mut parent.content, &mut focus.content, &mut target.content) {
                (
                    Content::Container(parent),
                    Content::View(focus_view),
                    Content::View(target_view),
                ) => {
                    let focus_pos = parent.children.iter().position(|id| focus_view.id == *id)?;
                    let target_pos = parent
                        .children
                        .iter()
                        .position(|id| target_view.id == *id)?;
                    // swap node positions so that traversal order is kept
                    parent.children[focus_pos] = target_view.id;
                    parent.children[target_pos] = focus_view.id;
                    // swap area so that views rendered at the correct location
                    std::mem::swap(&mut focus_view.area, &mut target_view.area);

                    Some(())
                }
                _ => unreachable!(),
            }
        } else {
            let [focus_parent, target_parent, focus, target] =
                self.nodes
                    .get_disjoint_mut([focus_parent, target_parent, focus, target])?;
            match (
                &mut focus_parent.content,
                &mut target_parent.content,
                &mut focus.content,
                &mut target.content,
            ) {
                (
                    Content::Container(focus_parent),
                    Content::Container(target_parent),
                    Content::View(focus_view),
                    Content::View(target_view),
                ) => {
                    let focus_pos = focus_parent
                        .children
                        .iter()
                        .position(|id| focus_view.id == *id)?;
                    let target_pos = target_parent
                        .children
                        .iter()
                        .position(|id| target_view.id == *id)?;
                    // re-parent target and focus nodes
                    std::mem::swap(
                        &mut focus_parent.children[focus_pos],
                        &mut target_parent.children[target_pos],
                    );
                    std::mem::swap(&mut focus.parent, &mut target.parent);
                    // swap area so that views rendered at the correct location
                    std::mem::swap(&mut focus_view.area, &mut target_view.area);

                    Some(())
                }
                _ => unreachable!(),
            }
        }
    }

    #[inline(always)]
    pub fn area(&self, tab: TabId) -> Rect {
        self.get_tree(tab).area
    }
}

#[derive(Debug)]
pub struct Traverse<'a> {
    tabs: &'a Tabs,
    stack: Vec<ViewId>, // TODO: reuse the one we use on update
}

impl<'a> Traverse<'a> {
    fn new(tabs: &'a Tabs, tree: &'a Tree) -> Self {
        Self {
            tabs,
            stack: vec![tree.root],
        }
    }
}

impl<'a> Iterator for Traverse<'a> {
    type Item = (ViewId, &'a View);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let key = self.stack.pop()?;

            let node = &self.tabs.nodes[key];

            match &node.content {
                Content::View(view) => return Some((key, view)),
                Content::Container(container) => {
                    self.stack.extend(container.children.iter().rev());
                }
            }
        }
    }
}

impl<'a> DoubleEndedIterator for Traverse<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let key = self.stack.pop()?;

            let node = &self.tabs.nodes[key];

            match &node.content {
                Content::View(view) => return Some((key, view)),
                Content::Container(container) => {
                    self.stack.extend(container.children.iter());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::editor::GutterConfig;
    use crate::DocumentId;

    #[test]
    fn find_split_in_direction() {
        let mut tabs = Tabs::new(Rect {
            x: 0,
            y: 0,
            width: 180,
            height: 80,
        });
        let mut tree = tabs.curr_tree_mut();
        let mut view = View::new(DocumentId::default(), GutterConfig::default());
        view.area = Rect::new(0, 0, 180, 80);
        tree.insert(view);

        let l0 = tree.focused();
        let view = View::new(DocumentId::default(), GutterConfig::default());
        tree.split(view, Layout::Vertical);
        let r0 = tree.focused();

        tree.set_focused(l0);
        let view = View::new(DocumentId::default(), GutterConfig::default());
        tree.split(view, Layout::Horizontal);
        let l1 = tree.focused();

        tree.set_focused(l0);
        let view = View::new(DocumentId::default(), GutterConfig::default());
        tree.split(view, Layout::Vertical);

        // Tree in test
        // | L0  | L2 |    |
        // |    L1    | R0 |
        let l2 = tree.focused();
        assert_eq!(Some(l0), tree.find_split_in_direction(l2, Direction::Left));
        assert_eq!(Some(l1), tree.find_split_in_direction(l2, Direction::Down));
        assert_eq!(Some(r0), tree.find_split_in_direction(l2, Direction::Right));
        assert_eq!(None, tree.find_split_in_direction(l2, Direction::Up));

        tree.set_focused(l1);
        assert_eq!(None, tree.find_split_in_direction(l1, Direction::Left));
        assert_eq!(None, tree.find_split_in_direction(l1, Direction::Down));
        assert_eq!(Some(r0), tree.find_split_in_direction(l1, Direction::Right));
        assert_eq!(Some(l0), tree.find_split_in_direction(l1, Direction::Up));

        tree.set_focused(l0);
        assert_eq!(None, tree.find_split_in_direction(l0, Direction::Left));
        assert_eq!(Some(l1), tree.find_split_in_direction(l0, Direction::Down));
        assert_eq!(Some(l2), tree.find_split_in_direction(l0, Direction::Right));
        assert_eq!(None, tree.find_split_in_direction(l0, Direction::Up));

        tree.set_focused(r0);
        assert_eq!(Some(l2), tree.find_split_in_direction(r0, Direction::Left));
        assert_eq!(None, tree.find_split_in_direction(r0, Direction::Down));
        assert_eq!(None, tree.find_split_in_direction(r0, Direction::Right));
        assert_eq!(None, tree.find_split_in_direction(r0, Direction::Up));
    }

    #[test]
    fn swap_split_in_direction() {
        let mut tabs = Tabs::new(Rect {
            x: 0,
            y: 0,
            width: 180,
            height: 80,
        });

        let doc_l0 = DocumentId::default();
        let mut view = View::new(doc_l0, GutterConfig::default());
        view.area = Rect::new(0, 0, 180, 80);
        let mut tree = tabs.curr_tree_mut();
        tree.insert(view);
        let l0 = tree.focused();

        let doc_r0 = DocumentId::default();
        let view = View::new(doc_r0, GutterConfig::default());
        tree.split(view, Layout::Vertical);
        let r0 = tree.focused();

        tree.set_focused(l0);

        let doc_l1 = DocumentId::default();
        let view = View::new(doc_l1, GutterConfig::default());
        tree.split(view, Layout::Horizontal);
        let l1 = tree.focused();
        tree.set_focused(l0);

        let doc_l2 = DocumentId::default();
        let view = View::new(doc_l2, GutterConfig::default());
        tree.split(view, Layout::Vertical);
        let l2 = tree.focused();

        // Views in test
        // | L0  | L2 |    |
        // |    L1    | R0 |

        // Document IDs in test
        // | l0  | l2 |    |
        // |    l1    | r0 |

        fn doc_id<'a>(tree: &TabProxyMut<'a>, view_id: ViewId) -> Option<DocumentId> {
            if let Content::View(view) = &tree.tabs.nodes[view_id].content {
                Some(view.doc)
            } else {
                None
            }
        }

        tree.set_focused(l0);
        // `*` marks the view in focus from view table (here L0)
        // | l0*  | l2 |    |
        // |    l1     | r0 |
        tree.swap_split_in_direction(Direction::Down);
        // | l1   | l2 |    |
        // |    l0*    | r0 |
        assert_eq!(tree.focused(), l0);
        assert_eq!(doc_id(&tree, l0), Some(doc_l1));
        assert_eq!(doc_id(&tree, l1), Some(doc_l0));
        assert_eq!(doc_id(&tree, l2), Some(doc_l2));
        assert_eq!(doc_id(&tree, r0), Some(doc_r0));

        tree.swap_split_in_direction(Direction::Right);

        // | l1  | l2 |     |
        // |    r0    | l0* |
        assert_eq!(tree.focused(), l0);
        assert_eq!(doc_id(&tree, l0), Some(doc_l1));
        assert_eq!(doc_id(&tree, l1), Some(doc_r0));
        assert_eq!(doc_id(&tree, l2), Some(doc_l2));
        assert_eq!(doc_id(&tree, r0), Some(doc_l0));

        // cannot swap, nothing changes
        tree.swap_split_in_direction(Direction::Up);
        // | l1  | l2 |     |
        // |    r0    | l0* |
        assert_eq!(tree.focused(), l0);
        assert_eq!(doc_id(&tree, l0), Some(doc_l1));
        assert_eq!(doc_id(&tree, l1), Some(doc_r0));
        assert_eq!(doc_id(&tree, l2), Some(doc_l2));
        assert_eq!(doc_id(&tree, r0), Some(doc_l0));

        // cannot swap, nothing changes
        tree.swap_split_in_direction(Direction::Down);
        // | l1  | l2 |     |
        // |    r0    | l0* |
        assert_eq!(tree.focused(), l0);
        assert_eq!(doc_id(&tree, l0), Some(doc_l1));
        assert_eq!(doc_id(&tree, l1), Some(doc_r0));
        assert_eq!(doc_id(&tree, l2), Some(doc_l2));
        assert_eq!(doc_id(&tree, r0), Some(doc_l0));

        tree.set_focused(l2);
        // | l1  | l2* |    |
        // |    r0     | l0 |

        tree.swap_split_in_direction(Direction::Down);
        // | l1  | r0  |    |
        // |    l2*    | l0 |
        assert_eq!(tree.focused(), l2);
        assert_eq!(doc_id(&tree, l0), Some(doc_l1));
        assert_eq!(doc_id(&tree, l1), Some(doc_l2));
        assert_eq!(doc_id(&tree, l2), Some(doc_r0));
        assert_eq!(doc_id(&tree, r0), Some(doc_l0));

        tree.swap_split_in_direction(Direction::Up);
        // | l2* | r0 |    |
        // |    l1    | l0 |
        assert_eq!(tree.focused(), l2);
        assert_eq!(doc_id(&tree, l0), Some(doc_l2));
        assert_eq!(doc_id(&tree, l1), Some(doc_l1));
        assert_eq!(doc_id(&tree, l2), Some(doc_r0));
        assert_eq!(doc_id(&tree, r0), Some(doc_l0));
    }
}
