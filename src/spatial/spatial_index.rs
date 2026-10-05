use crate::geometry::Point;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox2D {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl BoundingBox2D {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: max_x.max(min_x),
            max_y: max_y.max(min_y),
        }
    }

    pub fn from_points(points: &[Point]) -> Self {
        if points.is_empty() {
            return Self::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = points[0].x;
        let mut min_y = points[0].y;
        let mut max_x = points[0].x;
        let mut max_y = points[0].y;

        for point in points.iter().skip(1) {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        Self::new(min_x, min_y, max_x, max_y)
    }

    pub fn center(&self) -> Point {
        Point::new(
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
            0.0,
        )
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    pub fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_x && point.x <= self.max_x &&
        point.y >= self.min_y && point.y <= self.max_y
    }

    pub fn intersects(&self, other: &BoundingBox2D) -> bool {
        self.min_x <= other.max_x && self.max_x >= other.min_x &&
        self.min_y <= other.max_y && self.max_y >= other.min_y
    }

    pub fn contains(&self, other: &BoundingBox2D) -> bool {
        self.min_x <= other.min_x && self.max_x >= other.max_x &&
        self.min_y <= other.min_y && self.max_y >= other.max_y
    }

    pub fn union(&self, other: &BoundingBox2D) -> BoundingBox2D {
        BoundingBox2D::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    pub fn intersection(&self, other: &BoundingBox2D) -> Option<BoundingBox2D> {
        if self.intersects(other) {
            Some(BoundingBox2D::new(
                self.min_x.max(other.min_x),
                self.min_y.max(other.min_y),
                self.max_x.min(other.max_x),
                self.max_y.min(other.max_y),
            ))
        } else {
            None
        }
    }

    pub fn expand(&self, margin: f64) -> BoundingBox2D {
        BoundingBox2D::new(
            self.min_x - margin,
            self.min_y - margin,
            self.max_x + margin,
            self.max_y + margin,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.width() < 1e-9 || self.height() < 1e-9
    }

    pub fn diagonal_length(&self) -> f64 {
        (self.width().powi(2) + self.height().powi(2)).sqrt()
    }
}

pub trait SpatialObject {
    fn bounding_box(&self) -> BoundingBox2D;
    fn id(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct SpatialIndexEntry<T: SpatialObject> {
    pub object: T,
    pub bounding_box: BoundingBox2D,
}

impl<T: SpatialObject> SpatialIndexEntry<T> {
    pub fn new(object: T) -> Self {
        let bounding_box = object.bounding_box();
        Self { object, bounding_box }
    }
}

#[derive(Debug, Clone)]
pub struct RTreeNode<T: SpatialObject> {
    pub bounding_box: BoundingBox2D,
    pub children: Vec<RTreeChild<T>>,
    pub is_leaf: bool,
    pub level: usize,
}

impl<T: SpatialObject> RTreeNode<T> {
    pub fn new(is_leaf: bool, level: usize) -> Self {
        Self {
            bounding_box: BoundingBox2D::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            is_leaf,
            level,
        }
    }

    pub fn calculate_bounding_box(&mut self) {
        if self.children.is_empty() {
            self.bounding_box = BoundingBox2D::new(0.0, 0.0, 0.0, 0.0);
            return;
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for child in &self.children {
            let bbox = child.bounding_box();
            min_x = min_x.min(bbox.min_x);
            min_y = min_y.min(bbox.min_y);
            max_x = max_x.max(bbox.max_x);
            max_y = max_y.max(bbox.max_y);
        }

        self.bounding_box = BoundingBox2D::new(min_x, min_y, max_x, max_y);
    }
}

#[derive(Debug, Clone)]
pub enum RTreeChild<T: SpatialObject> {
    Node(Rc<RefCell<RTreeNode<T>>>),
    Entry(SpatialIndexEntry<T>),
}

impl<T: SpatialObject> RTreeChild<T> {
    pub fn bounding_box(&self) -> BoundingBox2D {
        match self {
            RTreeChild::Node(node) => node.borrow().bounding_box.clone(),
            RTreeChild::Entry(entry) => entry.bounding_box.clone(),
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;

const MIN_ENTRIES_PER_NODE: usize = 2;
const MAX_ENTRIES_PER_NODE: usize = 8;

#[derive(Debug, Clone)]
pub struct RTree<T: SpatialObject> {
    pub root: Option<Rc<RefCell<RTreeNode<T>>>>,
    pub height: usize,
    pub num_entries: usize,
}

impl<T: SpatialObject + Clone> RTree<T> {
    pub fn new() -> Self {
        Self {
            root: None,
            height: 0,
            num_entries: 0,
        }
    }

    pub fn insert(&mut self, object: T) {
        let entry = SpatialIndexEntry::new(object);
        self.num_entries += 1;

        if self.root.is_none() {
            let mut root = RTreeNode::new(true, 1);
            root.children.push(RTreeChild::Entry(entry));
            root.calculate_bounding_box();
            self.root = Some(Rc::new(RefCell::new(root)));
            self.height = 1;
            return;
        }

        let leaf_level = self.height;
        let node = self.choose_leaf(Rc::clone(self.root.as_ref().unwrap()), &entry);

        let mut node_ref = node.borrow_mut();
        node_ref.children.push(RTreeChild::Entry(entry));
        node_ref.calculate_bounding_box();
        drop(node_ref);

        if node.borrow().children.len() > MAX_ENTRIES_PER_NODE {
            self.split_node(node);
        }
    }

    fn choose_leaf(
        &self,
        node: Rc<RefCell<RTreeNode<T>>>,
        entry: &SpatialIndexEntry<T>,
    ) -> Rc<RefCell<RTreeNode<T>>> {
        let node_ref = node.borrow();

        if node_ref.is_leaf {
            return Rc::clone(&node);
        }

        let mut best_node = None;
        let mut best_increase = f64::MAX;
        let mut best_area = f64::MAX;

        for child in &node_ref.children {
            let child_bbox = child.bounding_box();
            let expanded = child_bbox.union(&entry.bounding_box);
            let area_increase = expanded.area() - child_bbox.area();

            if area_increase < best_increase - 1e-9 {
                best_increase = area_increase;
                best_area = child_bbox.area();
                best_node = Some(Rc::clone(
                    match child {
                        RTreeChild::Node(n) => n,
                        _ => unreachable!(),
                    }
                ));
            } else if (area_increase - best_increase).abs() < 1e-9 {
                if child_bbox.area() < best_area {
                    best_node = Some(Rc::clone(
                        match child {
                            RTreeChild::Node(n) => n,
                            _ => unreachable!(),
                        }
                    ));
                    best_area = child_bbox.area();
                }
            }
        }

        drop(node_ref);
        self.choose_leaf(best_node.unwrap().clone(), entry)
    }

    fn split_node(&mut self, node: Rc<RefCell<RTreeNode<T>>>) {
        let entries: Vec<SpatialIndexEntry<T>> = {
            let node_ref = node.borrow();
            node_ref.children.iter()
                .filter_map(|child| {
                    if let RTreeChild::Entry(entry) = child {
                        Some(entry.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        let (group1, group2) = self.quadratic_split(&entries);

        {
            let mut node_ref = node.borrow_mut();
            node_ref.children.clear();

            for entry in group1 {
                node_ref.children.push(RTreeChild::Entry(entry));
            }
            node_ref.calculate_bounding_box();
        }

        let new_node = Rc::new(RefCell::new(RTreeNode::new(true, 1)));

        let mut new_node_ref = new_node.borrow_mut();
        for entry in group2 {
            new_node_ref.children.push(RTreeChild::Entry(entry));
        }
        new_node_ref.calculate_bounding_box();
        drop(new_node_ref);

        self.insert_in_parent(node, Some(new_node));
    }

    fn quadratic_split(&self, entries: &[SpatialIndexEntry<T>]) -> (Vec<SpatialIndexEntry<T>>, Vec<SpatialIndexEntry<T>>) {
        if entries.len() <= MIN_ENTRIES_PER_NODE {
            return (entries[..MIN_ENTRIES_PER_NODE].to_vec(), entries[MIN_ENTRIES_PER_NODE..].to_vec());
        }

        let mut group1 = Vec::new();
        let mut group2 = Vec::new();
        let mut candidates: Vec<SpatialIndexEntry<T>> = entries.to_vec();

        let mut seed1_idx = 0;
        let mut seed2_idx = 1;
        let mut max_waste = -f64::MAX;

        for i in 0..candidates.len() {
            for j in (i + 1)..candidates.len() {
                let bbox = candidates[i].bounding_box.union(&candidates[j].bounding_box);
                let waste = bbox.area() - candidates[i].bounding_box.area() - candidates[j].bounding_box.area();
                if waste > max_waste {
                    max_waste = waste;
                    seed1_idx = i;
                    seed2_idx = j;
                }
            }
        }

        group1.push(candidates.remove(seed1_idx));
        group2.push(candidates.remove(if seed2_idx > seed1_idx { seed2_idx - 1 } else { seed2_idx }));

        while !candidates.is_empty() && group1.len() < MAX_ENTRIES_PER_NODE - MIN_ENTRIES_PER_NODE {
            let next_idx = self.pick_next(&candidates, &group1, &group2);
            let next = candidates.remove(next_idx);

            let bbox1 = self.group_bounding_box(&group1);
            let bbox2 = self.group_bounding_box(&group2);

            let area1 = if bbox1.is_empty() { 1.0 } else { bbox1.area() };
            let area2 = if bbox2.is_empty() { 1.0 } else { bbox2.area() };

            let expansion1 = bbox1.union(&next.bounding_box).area() - area1;
            let expansion2 = bbox2.union(&next.bounding_box).area() - area2;

            if expansion1 < expansion2 - 1e-9 {
                group1.push(next);
            } else if expansion2 < expansion1 - 1e-9 {
                group2.push(next);
            } else if area1 < area2 {
                group1.push(next);
            } else if area2 < area1 {
                group2.push(next);
            } else if group1.len() < MAX_ENTRIES_PER_NODE - MIN_ENTRIES_PER_NODE {
                group1.push(next);
            } else {
                group2.push(next);
            }
        }

        if !candidates.is_empty() {
            group2.extend(candidates);
        }

        (group1, group2)
    }

    fn pick_next(
        &self,
        candidates: &[SpatialIndexEntry<T>],
        group1: &[SpatialIndexEntry<T>],
        group2: &[SpatialIndexEntry<T>],
    ) -> usize {
        let bbox1 = self.group_bounding_box(group1);
        let bbox2 = self.group_bounding_box(group2);
        let area1 = if bbox1.is_empty() { 1.0 } else { bbox1.area() };
        let area2 = if bbox2.is_empty() { 1.0 } else { bbox2.area() };

        let mut max_diff = f64::MIN;
        let mut next_idx = 0;

        for (i, candidate) in candidates.iter().enumerate() {
            let expansion1 = bbox1.union(&candidate.bounding_box).area() - area1;
            let expansion2 = bbox2.union(&candidate.bounding_box).area() - area2;
            let diff = (expansion1 - expansion2).abs();

            if diff > max_diff {
                max_diff = diff;
                next_idx = i;
            }
        }

        next_idx
    }

    fn group_bounding_box(&self, entries: &[SpatialIndexEntry<T>]) -> BoundingBox2D {
        if entries.is_empty() {
            return BoundingBox2D::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for entry in entries {
            min_x = min_x.min(entry.bounding_box.min_x);
            min_y = min_y.min(entry.bounding_box.min_y);
            max_x = max_x.max(entry.bounding_box.max_x);
            max_y = max_y.max(entry.bounding_box.max_y);
        }

        BoundingBox2D::new(min_x, min_y, max_x, max_y)
    }

    fn insert_in_parent(
        &mut self,
        node: Rc<RefCell<RTreeNode<T>>>,
        new_node: Option<Rc<RefCell<RTreeNode<T>>>>,
    ) {
        let parent_level = node.borrow().level;

        if let Some(ref root) = self.root {
            if Rc::ptr_eq(root, &node) {
                if let Some(new_n) = new_node {
                    let mut new_root = RTreeNode::new(false, 0);
                    new_root.children.push(RTreeChild::Node(node));
                    new_root.children.push(RTreeChild::Node(new_n));
                    new_root.calculate_bounding_box();
                    self.root = Some(Rc::new(RefCell::new(new_root)));
                    self.height += 1;
                }
                return;
            }
        }
    }

    pub fn query_point(&self, point: &Point) -> Vec<T> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            self.query_point_recursive(Rc::clone(root), point, &mut results);
        }
        results
    }

    fn query_point_recursive(
        &self,
        node: Rc<RefCell<RTreeNode<T>>>,
        point: &Point,
        results: &mut Vec<T>,
    ) {
        let node_ref = node.borrow();

        if !node_ref.bounding_box.contains_point(point) {
            return;
        }

        for child in &node_ref.children {
            if child.bounding_box().contains_point(point) {
                match child {
                    RTreeChild::Entry(entry) => {
                        results.push(entry.object.clone());
                    }
                    RTreeChild::Node(child_node) => {
                        self.query_point_recursive(Rc::clone(child_node), point, results);
                    }
                }
            }
        }
    }

    pub fn query_bbox(&self, bbox: &BoundingBox2D) -> Vec<T> {
        let mut results = Vec::new();
        if let Some(ref root) = self.root {
            self.query_bbox_recursive(Rc::clone(root), bbox, &mut results);
        }
        results
    }

    fn query_bbox_recursive(
        &self,
        node: Rc<RefCell<RTreeNode<T>>>,
        query_bbox: &BoundingBox2D,
        results: &mut Vec<T>,
    ) {
        let node_ref = node.borrow();

        if !node_ref.bounding_box.intersects(query_bbox) {
            return;
        }

        for child in &node_ref.children {
            if child.bounding_box().intersects(query_bbox) {
                match child {
                    RTreeChild::Entry(entry) => {
                        if query_bbox.intersects(&entry.bounding_box) {
                            results.push(entry.object.clone());
                        }
                    }
                    RTreeChild::Node(child_node) => {
                        self.query_bbox_recursive(Rc::clone(child_node), query_bbox, results);
                    }
                }
            }
        }
    }

    pub fn query_radius(&self, center: &Point, radius: f64) -> Vec<T> {
        let query_bbox = BoundingBox2D::new(
            center.x - radius,
            center.y - radius,
            center.x + radius,
            center.y + radius,
        );

        self.query_bbox(&query_bbox)
            .into_iter()
            .filter(|obj| {
                let bbox = obj.bounding_box();
                let closest_x = bbox.min_x.max(center.x.min(bbox.max_x));
                let closest_y = bbox.min_y.max(center.y.min(bbox.max_y));
                let dx = center.x - closest_x;
                let dy = center.y - closest_y;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .collect()
    }

    pub fn nearest_neighbor(&self, point: &Point) -> Option<T> {
        let point_bbox = BoundingBox2D::new(point.x, point.y, point.x, point.y).expand(1e6);
        let candidates = self.query_bbox(&point_bbox);
        candidates.into_iter()
            .min_by(|a, b| {
                let bbox_a = a.bounding_box();
                let bbox_b = b.bounding_box();
                let dist_a = bbox_a.center().distance_to(point);
                let dist_b = bbox_b.center().distance_to(point);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.height = 0;
        self.num_entries = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn len(&self) -> usize {
        self.num_entries
    }
}

impl<T: SpatialObject + Clone> Default for RTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct QuadtreeNode<T: SpatialObject> {
    pub bounding_box: BoundingBox2D,
    pub capacity: usize,
    pub objects: Vec<SpatialIndexEntry<T>>,
    pub children: Option<Box<[QuadtreeNode<T>; 4]>>,
    pub is_leaf: bool,
}

impl<T: SpatialObject + Clone> QuadtreeNode<T> {
    pub fn new(bounding_box: BoundingBox2D) -> Self {
        Self {
            bounding_box,
            capacity: 4,
            objects: Vec::new(),
            children: None,
            is_leaf: true,
        }
    }

    pub fn insert(&mut self, object: T) -> bool {
        let entry = SpatialIndexEntry::new(object);

        if !self.bounding_box.contains(&entry.bounding_box) {
            return false;
        }

        if self.is_leaf {
            if self.objects.len() < self.capacity || self.bounding_box.width() < 1e-6 {
                self.objects.push(entry);
                return true;
            }
            self.subdivide();
        }

        for child in self.children.as_mut().unwrap().iter_mut() {
            if child.insert(entry.object.clone()) {
                return true;
            }
        }

        self.objects.push(entry);
        true
    }

    fn subdivide(&mut self) {
        let center = self.bounding_box.center();
        let half_width = self.bounding_box.width() / 2.0;
        let half_height = self.bounding_box.height() / 2.0;

        let nw = BoundingBox2D::new(
            self.bounding_box.min_x,
            center.y,
            center.x,
            self.bounding_box.max_y,
        );
        let ne = BoundingBox2D::new(
            center.x,
            center.y,
            self.bounding_box.max_x,
            self.bounding_box.max_y,
        );
        let sw = BoundingBox2D::new(
            self.bounding_box.min_x,
            self.bounding_box.min_y,
            center.x,
            center.y,
        );
        let se = BoundingBox2D::new(
            center.x,
            self.bounding_box.min_y,
            self.bounding_box.max_x,
            center.y,
        );

        let mut children = [
            QuadtreeNode::new(nw),
            QuadtreeNode::new(ne),
            QuadtreeNode::new(sw),
            QuadtreeNode::new(se),
        ];

        for obj in self.objects.drain(..) {
            let mut inserted = false;
            for child in children.iter_mut() {
                if child.bounding_box.contains(&obj.bounding_box) {
                    if child.insert(obj.object.clone()) {
                        inserted = true;
                        break;
                    }
                }
            }
            if !inserted {
                children[3].objects.push(obj);
            }
        }

        self.children = Some(Box::new(children));
        self.is_leaf = false;
    }

    pub fn query_point(&self, point: &Point) -> Vec<T> {
        let mut results = Vec::new();
        self.query_point_recursive(point, &mut results);
        results
    }

    fn query_point_recursive(&self, point: &Point, results: &mut Vec<T>) {
        if !self.bounding_box.contains_point(point) {
            return;
        }

        for obj in &self.objects {
            if obj.bounding_box.contains_point(point) {
                results.push(obj.object.clone());
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_point_recursive(point, results);
            }
        }
    }

    pub fn query_bbox(&self, query_bbox: &BoundingBox2D) -> Vec<T> {
        let mut results = Vec::new();
        self.query_bbox_recursive(query_bbox, &mut results);
        results
    }

    fn query_bbox_recursive(&self, query_bbox: &BoundingBox2D, results: &mut Vec<T>) {
        if !self.bounding_box.intersects(query_bbox) {
            return;
        }

        for obj in &self.objects {
            if obj.bounding_box.intersects(query_bbox) {
                results.push(obj.object.clone());
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_bbox_recursive(query_bbox, results);
            }
        }
    }

    pub fn query_radius(&self, center: &Point, radius: f64) -> Vec<T> {
        let query_bbox = BoundingBox2D::new(
            center.x - radius,
            center.y - radius,
            center.x + radius,
            center.y + radius,
        );

        self.query_bbox(&query_bbox)
            .into_iter()
            .filter(|obj| {
                let bbox = obj.bounding_box();
                let closest_x = bbox.min_x.max(center.x.min(bbox.max_x));
                let closest_y = bbox.min_y.max(center.y.min(bbox.max_y));
                let dx = center.x - closest_x;
                let dy = center.y - closest_y;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .collect()
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.children = None;
        self.is_leaf = true;
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && 
        self.children.as_ref().map_or(true, |c| c.iter().all(|child| child.is_empty()))
    }

    pub fn count(&self) -> usize {
        let mut count = self.objects.len();
        if let Some(ref children) = self.children {
            for child in children.iter() {
                count += child.count();
            }
        }
        count
    }

    pub fn height(&self) -> usize {
        if self.is_leaf {
            return 1;
        }
        if let Some(ref children) = self.children {
            1 + children.iter().map(|c| c.height()).max().unwrap_or(0)
        } else {
            1
        }
    }
}

#[derive(Debug, Clone)]
pub struct GridIndex<T: SpatialObject> {
    pub cell_size: f64,
    pub grid: HashMap<(i32, i32), Vec<SpatialIndexEntry<T>>>,
    pub bounding_box: BoundingBox2D,
    pub num_cells_x: i32,
    pub num_cells_y: i32,
}

impl<T: SpatialObject + Clone> GridIndex<T> {
    pub fn new(bounding_box: BoundingBox2D, cell_size: f64) -> Self {
        let num_cells_x = ((bounding_box.width() / cell_size).ceil() as i32).max(1);
        let num_cells_y = ((bounding_box.height() / cell_size).ceil() as i32).max(1);

        Self {
            cell_size,
            grid: HashMap::new(),
            bounding_box,
            num_cells_x,
            num_cells_y,
        }
    }

    pub fn insert(&mut self, object: T) {
        let entry = SpatialIndexEntry::new(object);
        let bbox = entry.bounding_box;

        let min_cell_x = ((bbox.min_x - self.bounding_box.min_x) / self.cell_size).floor() as i32;
        let max_cell_x = ((bbox.max_x - self.bounding_box.min_x) / self.cell_size).floor() as i32;
        let min_cell_y = ((bbox.min_y - self.bounding_box.min_y) / self.cell_size).floor() as i32;
        let max_cell_y = ((bbox.max_y - self.bounding_box.min_y) / self.cell_size).floor() as i32;

        for x in min_cell_x..=max_cell_x {
            for y in min_cell_y..=max_cell_y {
                self.grid.entry((x, y)).or_insert_with(Vec::new).push(entry.clone());
            }
        }
    }

    fn point_to_cell(&self, point: &Point) -> Option<(i32, i32)> {
        let cell_x = ((point.x - self.bounding_box.min_x) / self.cell_size).floor() as i32;
        let cell_y = ((point.y - self.bounding_box.min_y) / self.cell_size).floor() as i32;

        if cell_x >= 0 && cell_x < self.num_cells_x && cell_y >= 0 && cell_y < self.num_cells_y {
            Some((cell_x, cell_y))
        } else {
            None
        }
    }

    fn bbox_to_cells(&self, bbox: &BoundingBox2D) -> Vec<(i32, i32)> {
        let min_cell_x = ((bbox.min_x - self.bounding_box.min_x) / self.cell_size).floor() as i32;
        let max_cell_x = ((bbox.max_x - self.bounding_box.min_x) / self.cell_size).floor() as i32;
        let min_cell_y = ((bbox.min_y - self.bounding_box.min_y) / self.cell_size).floor() as i32;
        let max_cell_y = ((bbox.max_y - self.bounding_box.min_y) / self.cell_size).floor() as i32;

        let mut cells = Vec::new();
        for x in min_cell_x..=max_cell_x {
            for y in min_cell_y..=max_cell_y {
                if x >= 0 && x < self.num_cells_x && y >= 0 && y < self.num_cells_y {
                    cells.push((x, y));
                }
            }
        }
        cells
    }

    pub fn query_point(&self, point: &Point) -> Vec<T> {
        let mut results = Vec::new();

        if let Some(cell) = self.point_to_cell(point) {
            if let Some(objects) = self.grid.get(&cell) {
                for obj in objects {
                    if obj.bounding_box.contains_point(point) {
                        results.push(obj.object.clone());
                    }
                }
            }
        }

        results
    }

    pub fn query_bbox(&self, query_bbox: &BoundingBox2D) -> Vec<T> {
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for cell in self.bbox_to_cells(query_bbox) {
            if let Some(objects) = self.grid.get(&cell) {
                for obj in objects {
                    if !seen.contains(obj.object.id()) {
                        if obj.bounding_box.intersects(query_bbox) {
                            results.push(obj.object.clone());
                            seen.insert(obj.object.id());
                        }
                    }
                }
            }
        }

        results
    }

    pub fn query_radius(&self, center: &Point, radius: f64) -> Vec<T> {
        let query_bbox = BoundingBox2D::new(
            center.x - radius,
            center.y - radius,
            center.x + radius,
            center.y + radius,
        );

        self.query_bbox(&query_bbox)
            .into_iter()
            .filter(|obj| {
                let bbox = obj.bounding_box();
                let closest_x = bbox.min_x.max(center.x.min(bbox.max_x));
                let closest_y = bbox.min_y.max(center.y.min(bbox.max_y));
                let dx = center.x - closest_x;
                let dy = center.y - closest_y;
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .collect()
    }

    pub fn get_cell_contents(&self, cell_x: i32, cell_y: i32) -> Option<&Vec<SpatialIndexEntry<T>>> {
        self.grid.get(&(cell_x, cell_y))
    }

    pub fn get_all_cells(&self) -> Vec<((i32, i32), &Vec<SpatialIndexEntry<T>>)> {
        self.grid.iter()
            .filter(|(_, objects)| !objects.is_empty())
            .map(|(cell, objects)| (*cell, objects))
            .collect()
    }

    pub fn clear(&mut self) {
        self.grid.clear();
    }

    pub fn len(&self) -> usize {
        let mut count = 0;
        let mut seen = std::collections::HashSet::new();
        for objects in self.grid.values() {
            for obj in objects {
                if !seen.contains(obj.object.id()) {
                    count += 1;
                    seen.insert(obj.object.id());
                }
            }
        }
        count
    }

    pub fn is_empty(&self) -> bool {
        self.grid.values().all(|objects| objects.is_empty())
    }

    pub fn rebuild(&mut self, bounding_box: BoundingBox2D, cell_size: f64) {
        let entries: Vec<SpatialIndexEntry<T>> = {
            let mut entries = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for objects in self.grid.values() {
                for obj in objects {
                    if !seen.contains(obj.object.id()) {
                        entries.push(obj.clone());
                        seen.insert(obj.object.id());
                    }
                }
            }
            entries
        };

        *self = Self::new(bounding_box, cell_size);
        for entry in entries {
            self.insert(entry.object);
        }
    }
}

pub struct SpatialIndexFactory;

impl SpatialIndexFactory {
    pub fn create_rtree<T: SpatialObject + Clone>() -> RTree<T> {
        RTree::<T>::new()
    }

    pub fn create_quadtree<T: SpatialObject + Clone>(bounding_box: BoundingBox2D) -> QuadtreeNode<T> {
        QuadtreeNode::new(bounding_box)
    }

    pub fn create_grid<T: SpatialObject + Clone>(bounding_box: BoundingBox2D, cell_size: f64) -> GridIndex<T> {
        GridIndex::new(bounding_box, cell_size)
    }

    pub fn recommend_index<T: SpatialObject + Clone>(
        num_objects: usize,
        bounding_box: BoundingBox2D,
        query_pattern: QueryPattern,
    ) -> RecommendedIndex {
        if num_objects < 100 {
            RecommendedIndex::Quadtree(bounding_box)
        } else {
            match query_pattern {
                QueryPattern::PointQuery => RecommendedIndex::Grid(bounding_box, bounding_box.width() / 100.0),
                QueryPattern::RangeQuery => RecommendedIndex::RTree,
                QueryPattern::Mixed => RecommendedIndex::RTree,
                QueryPattern::RadiusQuery => RecommendedIndex::RTree,
                QueryPattern::KNNQuery => RecommendedIndex::RTree,
            }
        }
    }
}

pub enum RecommendedIndex {
    RTree,
    Quadtree(BoundingBox2D),
    Grid(BoundingBox2D, f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QueryPattern {
    PointQuery,
    RangeQuery,
    Mixed,
    RadiusQuery,
    KNNQuery,
}
