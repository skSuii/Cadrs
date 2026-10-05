//! 操作式撤销/重做历史：以实体为单位的增量操作栈。
//! Added(entity) / Removed(entity) / Modified(old, new) 三种操作，
//! push 后可 undo(&mut Document) / redo(&mut Document)。

use crate::data_structure::{Document, Entity};

/// 历史操作（持有完整实体快照）
#[derive(Debug, Clone)]
pub enum HistoryOp {
    /// 新增了实体（undo 移除，redo 重新插入，保留原 id）
    Added(Entity),
    /// 移除了实体（undo 重新插入，redo 再次移除）
    Removed(Entity),
    /// 修改了实体：(旧实体, 新实体)。undo 恢复旧值，redo 应用新值
    Modified(Entity, Entity),
}

/// 操作式历史栈（undo/redo 双栈，容量上限可配置，默认 200）
#[derive(Debug)]
pub struct OpHistory {
    undo_stack: Vec<HistoryOp>,
    redo_stack: Vec<HistoryOp>,
    limit: usize,
}

impl Default for OpHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl OpHistory {
    pub const DEFAULT_LIMIT: usize = 200;

    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            limit: Self::DEFAULT_LIMIT,
        }
    }

    /// 指定撤销栈容量上限的历史
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            limit,
        }
    }

    /// 撤销栈容量上限
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// 压入一条操作；超过容量时丢弃最旧的操作；清空 redo 栈
    pub fn push(&mut self, op: HistoryOp) {
        self.redo_stack.clear();
        self.undo_stack.push(op);
        while self.undo_stack.len() > self.limit {
            self.undo_stack.remove(0);
        }
    }

    /// 撤销最近一条操作；成功返回 true（栈空或操作无法应用返回 false）
    pub fn undo(&mut self, doc: &mut Document) -> bool {
        let Some(op) = self.undo_stack.pop() else {
            return false;
        };
        let ok = match &op {
            HistoryOp::Added(e) => doc.remove_entity(e.id()),
            HistoryOp::Removed(e) => {
                doc.add_entity(e.clone());
                true
            }
            HistoryOp::Modified(old, _) => {
                doc.entities_mut().insert(old.id().clone(), old.clone()).is_some()
            }
        };
        if ok {
            self.redo_stack.push(op);
        }
        ok
    }

    /// 重做最近撤销的操作；成功返回 true
    pub fn redo(&mut self, doc: &mut Document) -> bool {
        let Some(op) = self.redo_stack.pop() else {
            return false;
        };
        let ok = match &op {
            HistoryOp::Added(e) => {
                doc.add_entity(e.clone());
                true
            }
            HistoryOp::Removed(e) => doc.remove_entity(e.id()),
            HistoryOp::Modified(_, new) => {
                doc.entities_mut().insert(new.id().clone(), new.clone()).is_some()
            }
        };
        if ok {
            self.undo_stack.push(op);
        }
        ok
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 撤销栈是否为空（无任何可撤销操作）
    pub fn is_empty(&self) -> bool {
        self.undo_stack.is_empty()
    }

    /// 撤销栈中的操作数
    pub fn len(&self) -> usize {
        self.undo_stack.len()
    }

    /// 清空撤销与重做栈
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structure::{make_line, EntityGeometry};
    use crate::geometry::Point;

    #[test]
    fn test_op_history_undo_redo() {
        let mut doc = Document::new("test".to_string());
        let mut history = OpHistory::new();
        assert!(history.is_empty());
        assert!(!history.can_undo());

        let line = make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0));
        let id = line.id().clone();
        doc.add_entity(line.clone());
        history.push(HistoryOp::Added(line));

        assert!(history.can_undo());
        assert!(history.undo(&mut doc));
        assert!(!doc.entity_exists(&id));
        assert!(history.can_redo());
        assert!(history.redo(&mut doc));
        assert!(doc.entity_exists(&id));
    }

    #[test]
    fn test_op_history_modified() {
        let mut doc = Document::new("test".to_string());
        let mut history = OpHistory::new();

        let line = make_line(Point::new2d(0.0, 0.0), Point::new2d(10.0, 0.0));
        let id = line.id().clone();
        doc.add_entity(line.clone());

        let mut moved = line.clone();
        if let EntityGeometry::Line(l) = moved.geometry_mut() {
            l.start = Point::new2d(5.0, 5.0);
        }
        doc.entities_mut().insert(id.clone(), moved.clone());
        history.push(HistoryOp::Modified(line, moved));

        assert!(history.undo(&mut doc));
        match doc.get_entity(&id).unwrap().geometry() {
            EntityGeometry::Line(l) => assert_eq!(l.start.x, 0.0),
            _ => panic!("expected line"),
        }
        assert!(history.redo(&mut doc));
        match doc.get_entity(&id).unwrap().geometry() {
            EntityGeometry::Line(l) => assert_eq!(l.start.x, 5.0),
            _ => panic!("expected line"),
        }
    }

    #[test]
    fn test_op_history_limit() {
        let mut history = OpHistory::with_limit(3);
        for i in 0..5 {
            let line = make_line(
                Point::new2d(i as f64, 0.0),
                Point::new2d(i as f64 + 1.0, 0.0),
            );
            history.push(HistoryOp::Added(line));
        }
        assert_eq!(history.len(), 3);
    }
}
