use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub enum HotKeyError {
    Collision,
    CallbackError,
    CombinationNotFound
}

type HotKeyCallback<CallbackState, CallbackResult> = fn(&mut CallbackState) -> Result<CallbackResult, HotKeyError>;

pub struct HotKeyManager<TKey: PartialEq + Copy, CallbackState, CallbackResult> {
    graph: Graph<TKey, CallbackState, CallbackResult, HotKeyCallback<CallbackState, CallbackResult>>,
    comb_buf: Vec<TKey>,
}

impl<TKey: PartialEq + Copy, CallbackState, CallbackResult> HotKeyManager<TKey, CallbackState, CallbackResult> {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            comb_buf: vec![],
        }
    }

    pub fn register(&mut self, combination: &[TKey], callback: HotKeyCallback<CallbackState, CallbackResult>) -> Result<(), HotKeyError> {
        self.graph.write(combination, callback)
    }

    pub fn push(&mut self, payload: TKey) -> Result<(), HotKeyError> {
        self.comb_buf.push(payload);
        Ok(())
    }

    pub fn invoke_if_matched(&mut self, state: &mut CallbackState) -> Result<CallbackResult, HotKeyError> {
        if !self.graph.is_possible_path(&self.comb_buf) {
            return Err(HotKeyError::CombinationNotFound)
        }
        return self.graph.invoke(&self.comb_buf, state);
    }
}

trait GraphCallback<State, TResult>: FnMut(&mut State) -> Result<TResult, HotKeyError> {}

impl<CallbackState, CallbackResult, F> GraphCallback<CallbackState, CallbackResult> for F
where F: FnMut(&mut CallbackState) -> Result<CallbackResult, HotKeyError>
{
}

struct Graph<TKey: PartialEq + Copy, CallbackState, CallbackResult, Callback: GraphCallback<CallbackState, CallbackResult>> {
    begin: GraphNode<TKey, CallbackState, CallbackResult, Callback>
}

#[derive(Debug)]
enum GraphNode<TKey, CallbackState, CallbackResult, Callback: GraphCallback<CallbackState, CallbackResult>> {
    Node(
        Option<TKey>,
        Box<Vec<GraphNode<TKey, CallbackState, CallbackResult, Callback>>>,
        PhantomData<CallbackState>,
        PhantomData<CallbackResult>,
    ),
    List(TKey, Callback),
}

impl<TKey: PartialEq + Copy, CallbackState, CallbackResult, Callback: GraphCallback<CallbackState, CallbackResult>> Graph<TKey, CallbackState, CallbackResult, Callback> {
    pub fn new() -> Self {
        Self {
            begin: GraphNode::Node(None, Box::new(vec![]), PhantomData, PhantomData)
        }
    }

    pub fn write(&mut self, payload: &[TKey], callback: Callback) -> Result<(), HotKeyError> {
        let mut callback = Some(callback);
        let _ = Self::_write(&mut self.begin, payload, &mut callback)?;
        return Ok(())
    }

    fn _write(content: &mut GraphNode<TKey, CallbackState, CallbackResult, Callback>, payload: &[TKey], callback: &mut Option<Callback>) -> Result<bool, HotKeyError> {
        match content {
            GraphNode::Node(value, graph_nodes, _, _) => {
                if *value != None && *value != Some(payload[0]) {
                    return Ok(false)
                }

                let tail = if *value == None { payload } else { &payload[1..] };
                
                for gn in graph_nodes.iter_mut() {
                    if Self::_write(gn, tail, callback)? {
                        return Ok(true);
                    }
                }
                if tail.len() > 1 {
                    let node = GraphNode::Node(Some(tail[0]), Box::new(vec![]), PhantomData, PhantomData);
                    graph_nodes.push(node);
                    let node = graph_nodes.last_mut().unwrap();
                    return Self::_write(node, tail, callback)
                } else {
                    let callback = callback.take().unwrap();
                    let node = GraphNode::List(tail[0], callback);
                    graph_nodes.push(node);
                    return Ok(true);
                }
            },
            GraphNode::List(value, _) => {
                if payload.len() > 0 && *value == payload[0] {
                    return Err(HotKeyError::Collision);
                }
                return Ok(false)
            },
        }
    }

    pub fn is_possible_path(&self, payload: &[TKey]) -> bool {
        if payload.len() == 0 {
            return true;
        }
        Self::_is_possible_path(&self.begin, payload)
    }

    fn _is_possible_path(node: &GraphNode<TKey, CallbackState, CallbackResult, Callback>, payload: &[TKey]) -> bool {
        match node {
            GraphNode::Node(value, graph_nodes, _, _) => {
                if let Some(v) = *value {
                    if v != payload[0] {
                        return false;
                    }
                }
                let tail = if *value == None { payload } else { &payload[1..] };

                if tail.len() > 0 {
                    for gn in graph_nodes.iter() {
                        if Self::_is_possible_path(gn, tail) {
                            return true;
                        }
                    }
                    return false;
                }
                return true;
            },
            GraphNode::List(value, _) => {
                return payload.len() == 1 && *value == payload[0]
            },
        }
    }

    pub fn invoke(&mut self, payload: &[TKey], state: &mut CallbackState) -> Result<CallbackResult, HotKeyError> {
        if payload.is_empty() {
            return Err(HotKeyError::CombinationNotFound)
        }
        Self::_invoke(&mut self.begin, payload, state)
    }

    fn _invoke(node: &mut GraphNode<TKey, CallbackState, CallbackResult, Callback>, payload: &[TKey], state: &mut CallbackState) -> Result<CallbackResult, HotKeyError> {
        match node {
            GraphNode::Node(value, graph_nodes, _, _) => {
                if let Some(v) = *value {
                    if v != payload[0] {
                        return Err(HotKeyError::CombinationNotFound)
                    }
                }

                let tail = if *value == None { payload } else { &payload[1..] };

                if tail.len() > 0 {
                    for gn in graph_nodes.iter_mut() {
                        let r = Self::_invoke(gn, tail, state);
                        match r {
                            Err(HotKeyError::CombinationNotFound) => continue,
                            _ => return r
                        };
                    }
                    return Err(HotKeyError::CombinationNotFound)
                }

                return Err(HotKeyError::CombinationNotFound)
            },
            GraphNode::List(value, clbk) => {
                if payload.len() == 1 && *value == payload[0] {
                    return clbk(state)
                }
                Err(HotKeyError::CombinationNotFound)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hotkey_manager::{Graph, GraphCallback, GraphNode, HotKeyError};

    type TestCallback = fn(&mut ()) -> Result<(), HotKeyError>;

    fn noop(_: &mut ()) -> Result<(), HotKeyError> {
        Ok(())
    }

    mod graph {
        use std::marker::PhantomData;

        use crate::hotkey_manager::{Graph, GraphCallback, GraphNode, HotKeyError};
        use super::{TestCallback, noop};

        fn assert_graph(expected: &Graph<char, (), (), TestCallback>, actual: &Graph<char, (), (), TestCallback>) {
            _assert_node(&expected.begin, &actual.begin);
        }

        fn _assert_node(expected: &GraphNode<char, (), (), TestCallback>, actual: &GraphNode<char, (), (), TestCallback>) {
            match (expected, actual) {
                (GraphNode::List(exp_key, _), GraphNode::List(act_key, _)) => assert_eq!(exp_key, act_key),
                (GraphNode::Node(exp_key, exp_nodes, _, _), GraphNode::Node(act_key, act_nodes, _, _)) => {
                    assert_eq!(exp_key, act_key);
                    assert_eq!(exp_nodes.len(), act_nodes.len());
                    for i in 0..exp_nodes.len() {
                        _assert_node(&exp_nodes[i], &act_nodes[i]);
                    }
                }
                _ => panic!("nodes are not equal")
            }
        }

        #[test]
        fn should_add_single_node() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload = ['a'];
            assert_eq!(graph.write(&payload, noop), Ok(()));
            assert_graph(&Graph {
                begin: GraphNode::Node(None, Box::new(vec![
                    GraphNode::List('a', noop)
                ]), PhantomData, PhantomData)
            }, &graph);
        }

        #[test]
        fn should_add_single_sequence() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload = ['a', 'b', 'c'];
            assert_eq!(graph.write(&payload, noop), Ok(()));
            assert_graph(&Graph {
                begin: GraphNode::Node(None, Box::new(vec![
                    GraphNode::Node(Some('a'), Box::new(vec![
                        GraphNode::Node(Some('b'), Box::new(vec![
                            GraphNode::List('c', noop)
                        ]), PhantomData, PhantomData)
                    ]), PhantomData, PhantomData)
                ]), PhantomData, PhantomData)
            }, &graph);
        }

        #[test]
        fn should_add_sequences() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload1 = ['a', 'b', 'c'];
            let payload2 = ['a', 'b', 'v'];
            assert_eq!(graph.write(&payload1, noop), Ok(()));
            assert_eq!(graph.write(&payload2, noop), Ok(()));
            assert_graph(&Graph {
                begin: GraphNode::Node(None, Box::new(vec![
                    GraphNode::Node(Some('a'), Box::new(vec![
                        GraphNode::Node(Some('b'), Box::new(vec![
                            GraphNode::List('c', noop),
                            GraphNode::List('v', noop),
                        ]), PhantomData, PhantomData)
                    ]), PhantomData, PhantomData)
                ]), PhantomData, PhantomData)
            }, &graph);
        }

        #[test]
        fn should_add_different_sequences() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload1 = ['a', 'b', 'c'];
            let payload2 = ['q', 'w', 'e'];
            assert_eq!(graph.write(&payload1, noop), Ok(()));
            assert_eq!(graph.write(&payload2, noop), Ok(()));
            assert_graph(&Graph {
                begin: GraphNode::Node(None, Box::new(vec![
                    GraphNode::Node(Some('a'), Box::new(vec![
                        GraphNode::Node(Some('b'), Box::new(vec![
                            GraphNode::List('c', noop),
                        ]), PhantomData, PhantomData)
                    ]), PhantomData, PhantomData),
                    GraphNode::Node(Some('q'), Box::new(vec![
                        GraphNode::Node(Some('w'), Box::new(vec![
                            GraphNode::List('e', noop)
                        ]), PhantomData, PhantomData)
                    ]), PhantomData, PhantomData)
                ]), PhantomData, PhantomData)
            }, &graph);
        }

        #[test]
        fn should_fail_on_same_sequences() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload1 = ['a', 'b', 'c'];
            let payload2 = ['a', 'b', 'c'];
            assert_eq!(graph.write(&payload1, noop), Ok(()));
            assert_eq!(graph.write(&payload2, noop), Err(HotKeyError::Collision));
            assert_graph(&Graph {
                begin: GraphNode::Node(None, Box::new(vec![
                    GraphNode::Node(Some('a'), Box::new(vec![
                        GraphNode::Node(Some('b'), Box::new(vec![
                            GraphNode::List('c', noop),
                        ]), PhantomData, PhantomData)
                    ]), PhantomData, PhantomData)
                ]), PhantomData, PhantomData)
            }, &graph);
        }

        #[test]
        fn should_fail_on_same_substring() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload1 = ['a'];
            let payload2 = ['a', 'b'];
            assert_eq!(graph.write(&payload1, noop), Ok(()));
            assert_eq!(graph.write(&payload2, noop), Err(HotKeyError::Collision));
            assert_graph(&Graph {
                begin: GraphNode::Node(None, Box::new(vec![
                    GraphNode::List('a', noop)
                ]), PhantomData, PhantomData)
            }, &graph);
        }

        #[test]
        fn should_find_path() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload = ['a'];
            assert_eq!(graph.write(&payload, noop), Ok(()));
            assert!(graph.is_possible_path(&payload));
        }

        #[test]
        fn should_find_path_on_one_sequence() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload = ['a', 'b', 'c'];
            assert_eq!(graph.write(&payload, noop), Ok(()));
            assert!(graph.is_possible_path(&payload));
        }

        #[test]
        fn should_find_path_on_one_subsequence() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload = ['a', 'b', 'c'];
            assert_eq!(graph.write(&payload, noop), Ok(()));
            assert!(graph.is_possible_path(&payload[..2]));
        }

        #[test]
        fn should_find_path_on_several_subsequences() {
            let mut graph = Graph::<char, (), (), TestCallback>::new();
            let payload1 = ['a', 'b', 'c'];
            let payload2 = ['a', 'g', 'h', 'c'];
            assert_eq!(graph.write(&payload1, noop), Ok(()));
            assert_eq!(graph.write(&payload2, noop), Ok(()));
            assert!(graph.is_possible_path(&payload1[..1]));
            assert!(graph.is_possible_path(&payload1[..2]));
            assert!(graph.is_possible_path(&payload2[..3]));
        }
    }

    mod hotkey_manager {
        use crate::hotkey_manager::{HotKeyError, HotKeyManager};

        #[test]
        fn should_invoke() {
            let mut hk_manager = HotKeyManager::new();
            assert_eq!(
                hk_manager.register(&['a'], |_: &mut ()| {
                    Ok(Some(true))
                }),
                Ok(())
            );
            assert_eq!(hk_manager.push('a'), Ok(()));
            assert_eq!(hk_manager.invoke_if_matched(&mut ()), Ok(Some(true)))
        }

        #[test]
        fn should_throw_err_on_invoke() {
            let mut hk_manager = HotKeyManager::new();
            assert_eq!(
                hk_manager.register(&['a'], |_: &mut ()| {
                    Ok(Some(true))
                }),
                Ok(())
            );
            assert_eq!(hk_manager.push('b'), Ok(()));
            assert_eq!(hk_manager.invoke_if_matched(&mut ()), Err(HotKeyError::CombinationNotFound))
        }
    }
}

