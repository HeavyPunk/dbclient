#[derive(Debug, PartialEq)]
pub enum HotKeyError {
    Collision
}

pub struct HotKeyManager {
    graph: Graph<char>
}

impl HotKeyManager {
    pub fn new() -> Self {
        Self {
            graph: Graph::new()
        }
    }
    pub fn register(&mut self, combination: &[char]) -> Result<(), HotKeyError> {
        self.graph.write(combination)
    }
}


struct Graph<TKey> where TKey: PartialEq + Copy {
    begin: GraphNode<TKey>
}

#[derive(Debug)]
enum GraphNode<TKey> {
    Node(Option<TKey>, Box<Vec<GraphNode<TKey>>>),
    List(TKey),
}

impl<TKey> Graph<TKey> where TKey: PartialEq + Copy {
    pub fn new() -> Self {
        Self {
            begin: GraphNode::Node(None, Box::new(vec![]))
        }
    }

    pub fn write(&mut self, payload: &[TKey]) -> Result<(), HotKeyError> {
        let _ = Graph::<TKey>::_write(&mut self.begin, payload)?;
        return Ok(())
    }

    fn _write(content: &mut GraphNode<TKey>, payload: &[TKey]) -> Result<bool, HotKeyError> {
        match content {
            GraphNode::Node(value, graph_nodes) => {
                if *value != None && *value != Some(payload[0]) {
                    return Ok(false)
                }

                let tail = if *value == None { payload } else { &payload[1..] };
                
                for gn in graph_nodes.iter_mut() {
                    if Graph::<TKey>::_write(gn, tail)? {
                        return Ok(true);
                    }
                }
                if tail.len() > 1 {
                    let node = GraphNode::Node(Some(tail[0]), Box::new(vec![]));
                    graph_nodes.push(node);
                    let node = graph_nodes.last_mut().unwrap();
                    return Graph::<TKey>::_write(node, tail)
                } else {
                    let node = GraphNode::List(tail[0]);
                    graph_nodes.push(node);
                    return Ok(true);
                }
            },
            GraphNode::List(value) => {
                if payload.len() > 0 && *value == payload[0] {
                    return Err(HotKeyError::Collision);
                }
                return Ok(false)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hotkey_manager::{Graph, GraphNode, HotKeyError};

    fn assert_graph(expected: &Graph<char>, actual: &Graph<char>) {
        _assert_node(&expected.begin, &actual.begin);
    }

    fn _assert_node(expected: &GraphNode<char>, actual: &GraphNode<char>) {
        match (expected, actual) {
            (GraphNode::List(exp_key), GraphNode::List(act_key)) => assert_eq!(exp_key, act_key),
            (GraphNode::Node(exp_key, exp_nodes), GraphNode::Node(act_key, act_nodes)) => {
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
        let mut graph = Graph::<char>::new();
        let payload = ['a'];
        assert_eq!(graph.write(&payload), Ok(()));
        assert_graph(&Graph {
            begin: GraphNode::Node(None, Box::new(vec![
                GraphNode::List('a')
            ]))
        }, &graph);
    }

    #[test]
    fn should_add_single_sequence() {
        let mut graph = Graph::<char>::new();
        let payload = ['a', 'b', 'c'];
        assert_eq!(graph.write(&payload), Ok(()));
        assert_graph(&Graph {
            begin: GraphNode::Node(None, Box::new(vec![
                GraphNode::Node(Some('a'), Box::new(vec![
                    GraphNode::Node(Some('b'), Box::new(vec![
                        GraphNode::List('c')
                    ]))
                ]))
            ]))
        }, &graph);
    }

    #[test]
    fn should_add_sequences() {
        let mut graph = Graph::<char>::new();
        let payload1 = ['a', 'b', 'c'];
        let payload2 = ['a', 'b', 'v'];
        assert_eq!(graph.write(&payload1), Ok(()));
        assert_eq!(graph.write(&payload2), Ok(()));
        assert_graph(&Graph {
            begin: GraphNode::Node(None, Box::new(vec![
                GraphNode::Node(Some('a'), Box::new(vec![
                    GraphNode::Node(Some('b'), Box::new(vec![
                        GraphNode::List('c'),
                        GraphNode::List('v'),
                    ]))
                ]))
            ]))
        }, &graph);
    }

    #[test]
    fn should_add_different_sequences() {
        let mut graph = Graph::<char>::new();
        let payload1 = ['a', 'b', 'c'];
        let payload2 = ['q', 'w', 'e'];
        assert_eq!(graph.write(&payload1), Ok(()));
        assert_eq!(graph.write(&payload2), Ok(()));
        assert_graph(&Graph {
            begin: GraphNode::Node(None, Box::new(vec![
                GraphNode::Node(Some('a'), Box::new(vec![
                    GraphNode::Node(Some('b'), Box::new(vec![
                        GraphNode::List('c'),
                    ]))
                ])),
                GraphNode::Node(Some('q'), Box::new(vec![
                    GraphNode::Node(Some('w'), Box::new(vec![
                        GraphNode::List('e')
                    ]))
                ]))
            ]))
        }, &graph);
    }

    #[test]
    fn should_fail_on_same_sequences() {
        let mut graph = Graph::<char>::new();
        let payload1 = ['a', 'b', 'c'];
        let payload2 = ['a', 'b', 'c'];
        assert_eq!(graph.write(&payload1), Ok(()));
        assert_eq!(graph.write(&payload2), Err(HotKeyError::Collision));
        assert_graph(&Graph {
            begin: GraphNode::Node(None, Box::new(vec![
                GraphNode::Node(Some('a'), Box::new(vec![
                    GraphNode::Node(Some('b'), Box::new(vec![
                        GraphNode::List('c'),
                    ]))
                ]))
            ]))
        }, &graph);
    }

    #[test]
    fn should_fail_on_same_substring() {
        let mut graph = Graph::<char>::new();
        let payload1 = ['a'];
        let payload2 = ['a', 'b'];
        assert_eq!(graph.write(&payload1), Ok(()));
        assert_eq!(graph.write(&payload2), Err(HotKeyError::Collision));
        assert_graph(&Graph {
            begin: GraphNode::Node(None, Box::new(vec![
                GraphNode::List('a')
            ]))
        }, &graph);
    }
}

