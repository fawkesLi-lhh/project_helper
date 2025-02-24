import { useEffect, useState, useRef } from "react";
import "./App.scss";
import MermaidView from "./components/mermaid/MermaidView";
import {
  IMermaidEdgeDefinition,
  IMermaidNodeDefinition,
  MermaidChartDirection,
} from "./shared/models/mermaid.model";
import { Node, Edge } from "reactflow";
import ReactflowView from "./components/reactflow/ReactflowView";
import { v4 as uuidv4 } from "uuid";
import { MermaidParserEvent } from "./shared/models/mermaid.model";
import { Allotment } from "allotment";
import { editor } from "monaco-editor";
import { saveAs } from 'file-saver';

interface NodeData {
  id: string;
  name: string;
}

function App() {
  const [graphDefinition, setGraphDefinition] = useState(`flowchart TD`);
  const [nowSelectedNode, setNowSelectedNode] = useState<string | null>(null);
  const [reactflowNodes, setReactflowNodes] = useState<Node[]>([]);
  const [reactflowEdges, setReactflowEdges] = useState<Edge[]>([]);
  const [mermaidChartDirection, setMermaidChartDirection] =
    useState<MermaidChartDirection>(MermaidChartDirection.TD);
  const [editorInstance, setEditorInstance] =
    useState<editor.IStandaloneCodeEditor>();
  const [relatedNodes, setRelatedNodes] = useState<NodeData[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);

  useEffect(() => {
    const fetchRelatedNodes = async () => {
      try {
        // 如果nowSelectedNode为空，则不加related_node_id参数
        let url = "http://localhost:4096/node";
        url += `?hint_node_id=${searchQuery}`;
        if (nowSelectedNode) {
          url += `&&related_node_id=${nowSelectedNode}`;
        }
        const response = await fetch(url);
        const data = await response.json();
        console.log('relatedNodes', data);
        const ndata: NodeData[] = data.data;
        setRelatedNodes(ndata);
        //setRelatedNodes(data);
      } catch (error) {
        console.error("Error fetching related nodes:", error);
      }
    };

    fetchRelatedNodes();
  }, [nowSelectedNode, searchQuery]);
  // http://localhost:4096/init_graph
  // ... existing code ...
  useEffect(() => {
    // Function to call the API
    const initGraph = async () => {
      try {
        const response = await fetch("http://localhost:4096/init_graph");
        if (!response.ok) {
          throw new Error("Network response was not ok");
        }
        const data = await response.json();
        console.log("Graph initialized:", data);
        await refreshGraph();
      } catch (error) {
        console.error("Error initializing graph:", error);
      }
    };

    // Call the function when the component mounts
    initGraph();

  }, []); // Empty dependency array ensures this runs once on mount

  function handleMermaidDefinitionChange(event: MermaidParserEvent) {
    console.log('handleMermaidDefinitionChange', event);
    const reactflowEdges: Edge[] = event.edges.map(
      (mermaidEdge: IMermaidEdgeDefinition) =>
      ({
        id: uuidv4(),
        source: mermaidEdge.start,
        target: mermaidEdge.end,
        type: "customEdgeType",
        markerStart: "oneOrMany",
        markerEnd: "arrow-end",
        style: { stroke: "#f6ab6c" },
        elementsSelectable: true,
        label: mermaidEdge.text,
        // markerEnd: {
        //   type: MarkerType.ArrowClosed,
        // },
        animated: false,
        data: {
          label: mermaidEdge.text,
          raw: mermaidEdge,
        },
      } as Edge)
    ),
      reactflowNodes: Node[] = event.nodes.map(
        (mermaidNode: IMermaidNodeDefinition, index: number) => ({
          id: mermaidNode.id,
          position: { x: index * 200, y: index * 200 },
          type: "customNodeType",
          dragHandle: ".custom-node",
          data: {
            label: mermaidNode.text,
            raw: mermaidNode,
            layoutDirection: event.direction,
            onNodeClick: (nodeId: string) => setNowSelectedNode(nodeId),
          },
        })
      );

    console.log('reactflowNodes', reactflowNodes);
    console.log('reactflowEdges', reactflowEdges);
    setReactflowNodes(reactflowNodes);
    setReactflowEdges(reactflowEdges);
    setMermaidChartDirection(event.direction);
  }

  function resetEditorLayout(): void {
    editorInstance?.layout();
  }

  function onEditorInit(editor: editor.IStandaloneCodeEditor) {
    setEditorInstance(editor);

    resetEditorLayout();
  }

  const refreshGraph = async () => {
    let resp2 = await fetch("http://localhost:4096/graph");
    let data2 = await resp2.json();
    console.log('data2', data2);
    let graph = data2.data;
    console.log('graph', graph);
    setGraphDefinition(graph);
  }

  const handlePrintSelectedNodeId = async () => {
    if (selectedNodeId) {
      console.log('Selected selectedNodeId', selectedNodeId);
      try {
        let params = {
          id: selectedNodeId,
          hint_node_id: nowSelectedNode,
        }
        let url = "http://localhost:4096/node";
        let response = await fetch(url, {
          method: "PUT",
          body: JSON.stringify(params),
          headers: {
            'Content-Type': 'application/json'
          }
        });
        await refreshGraph();
      } catch (error) {
        console.error("Error fetching related nodes:", error);
      }
    } else {
      console.log("No node Selected");
    }
  };

  const handleFetchAndSaveGraph = async () => {
    try {
      const response = await fetch("http://localhost:4096/graph");
      if (!response.ok) {
        throw new Error("Network response was not ok");
      }
      const data = await response.json();
      const filename = prompt("Enter the filename to save the graph data:", "graphData");
      if (filename) {
        const blob = new Blob([data.data], { type: "application/json" });
        saveAs(blob, `${filename}.mermaid`);
      }
    } catch (error) {
      console.error("Error fetching graph data:", error);
    }
  };

  const handleDeleteSelectedNode = async () => {
    if (selectedNodeId) {
      try {
        const response = await fetch(`http://localhost:4096/node?id=${nowSelectedNode}`, {
          method: "DELETE",
        });
        if (!response.ok) {
          throw new Error("Network response was not ok");
        }
        console.log(`Node with id ${nowSelectedNode} deleted successfully.`);
        setNowSelectedNode(null); // Clear the selection after deletion
        await refreshGraph();
      } catch (error) {
        console.error("Error deleting node:", error);
      }
    } else {
      console.log("No node selected to delete.");
    }
  };

  return (
    <>
      {/* General Editor Layout */}
      <div className="editor-layout">
        <Allotment
          onChange={() => resetEditorLayout()}
          onDragEnd={() => resetEditorLayout()}
        >
          {/* Mermaid Side */}
          <Allotment.Pane minSize={500}>
            <div className="mermaid-editor">
              {/* <h1>Mermaid Editor</h1> */}

              {/* Monaco Editor Container */}
              {/* <div className="monaco-editor-container">
                <MonacoEditorView
                  code={graphDefinition}
                  onCodeChange={(event: string) => setGraphDefinition(event)}
                  onInit={(editor: editor.IStandaloneCodeEditor) =>
                    onEditorInit(editor)
                  }
                />
              </div> */}

              {/* Preview Container */}
              <div className="preview-container">
                <MermaidView
                  graphDefinition={graphDefinition}
                  onMermaidDefinitionChange={(event: MermaidParserEvent) =>
                    handleMermaidDefinitionChange(event)
                  }
                />
              </div>
            </div>
            <div className="react-flow-editor">
              {/* <h1>Reactflow Editor</h1> */}
              <div className="selected-node-display">
                <button onClick={handleFetchAndSaveGraph}>Fetch & Save Graph</button>
                <button onClick={() => setNowSelectedNode(null)}>Clear Selection</button>
                <button onClick={handlePrintSelectedNodeId}>PUT Node</button>
                <button onClick={handleDeleteSelectedNode}>Delete Node</button>
                Selected Node: {nowSelectedNode}
                <input
                  type="text"
                  placeholder="Search related nodes"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                />
                <select onChange={(e) => setSelectedNodeId(e.target.value)} value={selectedNodeId || ""}>
                  <option value="" disabled>Select a node</option>
                  {relatedNodes.map((node) => (
                    <option key={node.id} value={node.id}>
                      {node.name}
                    </option>
                  ))}
                </select>
              </div>
              <ReactflowView
                nodes={reactflowNodes}
                edges={reactflowEdges}
                direction={mermaidChartDirection}
              ></ReactflowView>
            </div>
          </Allotment.Pane>

          {/* Reactflow Side */}
          {/* <Allotment.Pane minSize={500}>
            
          </Allotment.Pane> */}
        </Allotment>
      </div>
    </>
  );
}

export default App;
