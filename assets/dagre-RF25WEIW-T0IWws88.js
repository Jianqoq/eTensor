import{m as w,t as a,bp as M,bq as j,bd as Y,br as H,bj as W,be as D,bc as $,bs as _,bt as q,bu as Q,bv as Z,bw as z,bx as K,W as U}from"./mermaid.esm.min-DOQkI20Q.js";import{f as V}from"./chunk-FASC7IG4-BY0rwTSS.js";import{m as J}from"./chunk-ZN7TASNU-BugEiHGY.js";import{s as x,_ as ee,J as k}from"./chunk-5ZJXQJOJ-HlTSsZNY.js";import"./app-HbYCe8KN.js";function X(e){var r={options:{directed:e.isDirected(),multigraph:e.isMultigraph(),compound:e.isCompound()},nodes:G(e),edges:P(e)};return x(e.graph())||(r.value=ee(e.graph())),r}w(X,"write");function G(e){return k(e.nodes(),function(r){var n=e.node(r),d=e.parent(r),l={v:r};return x(n)||(l.value=n),x(d)||(l.parent=d),l})}w(G,"writeNodes");function P(e){return k(e.edges(),function(r){var n=e.edge(r),d={v:r.v,w:r.w};return x(r.name)||(d.name=r.name),x(n)||(d.value=n),d})}w(P,"writeEdges");var c=new Map,E=new Map,B=new Map,ne=w(()=>{E.clear(),B.clear(),c.clear()},"clear"),I=w((e,r)=>{let n=E.get(r)||[];return a.trace("In isDescendant",r," ",e," = ",n.includes(e)),n.includes(e)},"isDescendant"),re=w((e,r)=>{let n=E.get(r)||[];return a.info("Descendants of ",r," is ",n),a.info("Edge is ",e),e.v===r||e.w===r?!1:n?n.includes(e.v)||I(e.v,r)||I(e.w,r)||n.includes(e.w):(a.debug("Tilt, ",r,",not in descendants"),!1)},"edgeInCluster"),A=w((e,r,n,d)=>{a.warn("Copying children of ",e,"root",d,"data",r.node(e),d);let l=r.children(e)||[];e!==d&&l.push(e),a.warn("Copying (nodes) clusterId",e,"nodes",l),l.forEach(o=>{if(r.children(o).length>0)A(o,r,n,d);else{let i=r.node(o);a.info("cp ",o," to ",d," with parent ",e),n.setNode(o,i),d!==r.parent(o)&&(a.warn("Setting parent",o,r.parent(o)),n.setParent(o,r.parent(o))),e!==d&&o!==e?(a.debug("Setting parent",o,e),n.setParent(o,e)):(a.info("In copy ",e,"root",d,"data",r.node(e),d),a.debug("Not Setting parent for node=",o,"cluster!==rootId",e!==d,"node!==clusterId",o!==e));let s=r.edges(o);a.debug("Copying Edges",s),s.forEach(u=>{a.info("Edge",u);let b=r.edge(u.v,u.w,u.name);a.info("Edge data",b,d);try{re(u,d)?(a.info("Copying as ",u.v,u.w,b,u.name),n.setEdge(u.v,u.w,b,u.name),a.info("newGraph edges ",n.edges(),n.edge(n.edges()[0]))):a.info("Skipping copy of edge ",u.v,"-->",u.w," rootId: ",d," clusterId:",e)}catch(N){a.error(N)}})}a.debug("Removing node",o),r.removeNode(o)})},"copy"),R=w((e,r)=>{let n=r.children(e),d=[...n];for(let l of n)B.set(l,e),d=[...d,...R(l,r)];return d},"extractDescendants"),te=w((e,r,n)=>{let d=e.edges().filter(s=>s.v===r||s.w===r),l=e.edges().filter(s=>s.v===n||s.w===n),o=d.map(s=>({v:s.v===r?n:s.v,w:s.w===r?r:s.w})),i=l.map(s=>({v:s.v,w:s.w}));return o.filter(s=>i.some(u=>s.v===u.v&&s.w===u.w))},"findCommonEdges"),S=w((e,r,n)=>{let d=r.children(e);if(a.trace("Searching children of id ",e,d),d.length<1)return e;let l;for(let o of d){let i=S(o,r,n),s=te(r,n,i);if(i)if(s.length>0)l=i;else return i}return l},"findNonClusterChild"),O=w(e=>!c.has(e)||!c.get(e).externalConnections?e:c.has(e)?c.get(e).id:e,"getAnchorId"),ae=w((e,r)=>{if(!e||r>10){a.debug("Opting out, no graph ");return}else a.debug("Opting in, graph ");e.nodes().forEach(function(n){e.children(n).length>0&&(a.warn("Cluster identified",n," Replacement id in edges: ",S(n,e,n)),E.set(n,R(n,e)),c.set(n,{id:S(n,e,n),clusterData:e.node(n)}))}),e.nodes().forEach(function(n){let d=e.children(n),l=e.edges();d.length>0?(a.debug("Cluster identified",n,E),l.forEach(o=>{let i=I(o.v,n),s=I(o.w,n);i^s&&(a.warn("Edge: ",o," leaves cluster ",n),a.warn("Descendants of XXX ",n,": ",E.get(n)),c.get(n).externalConnections=!0)})):a.debug("Not a cluster ",n,E)});for(let n of c.keys()){let d=c.get(n).id,l=e.parent(d);l!==n&&c.has(l)&&!c.get(l).externalConnections&&(c.get(n).id=l)}e.edges().forEach(function(n){let d=e.edge(n);a.warn("Edge "+n.v+" -> "+n.w+": "+JSON.stringify(n)),a.warn("Edge "+n.v+" -> "+n.w+": "+JSON.stringify(e.edge(n)));let l=n.v,o=n.w;if(a.warn("Fix XXX",c,"ids:",n.v,n.w,"Translating: ",c.get(n.v)," --- ",c.get(n.w)),c.get(n.v)||c.get(n.w)){if(a.warn("Fixing and trying - removing XXX",n.v,n.w,n.name),l=O(n.v),o=O(n.w),e.removeEdge(n.v,n.w,n.name),l!==n.v){let i=e.parent(l);c.get(i).externalConnections=!0,d.fromCluster=n.v}if(o!==n.w){let i=e.parent(o);c.get(i).externalConnections=!0,d.toCluster=n.w}a.warn("Fix Replacing with XXX",l,o,n.name),e.setEdge(l,o,d,n.name)}}),a.warn("Adjusted Graph",X(e)),T(e,0),a.trace(c)},"adjustClustersAndEdges"),T=w((e,r)=>{var l,o;if(a.warn("extractor - ",r,X(e),e.children("D")),r>10){a.error("Bailing out");return}let n=e.nodes(),d=!1;for(let i of n){let s=e.children(i);d=d||s.length>0}if(!d){a.debug("Done, no node has children",e.nodes());return}a.debug("Nodes = ",n,r);for(let i of n)if(a.debug("Extracting node",i,c,c.has(i)&&!c.get(i).externalConnections,!e.parent(i),e.node(i),e.children("D")," Depth ",r),!c.has(i))a.debug("Not a cluster",i,r);else if(!c.get(i).externalConnections&&e.children(i)&&e.children(i).length>0){a.warn("Cluster without external connections, without a parent and with children",i,r);let s=e.graph().rankdir==="TB"?"LR":"TB";(o=(l=c.get(i))==null?void 0:l.clusterData)!=null&&o.dir&&(s=c.get(i).clusterData.dir,a.warn("Fixing dir",c.get(i).clusterData.dir,s));let u=new J({multigraph:!0,compound:!0}).setGraph({rankdir:s,nodesep:50,ranksep:50,marginx:8,marginy:8}).setDefaultEdgeLabel(function(){return{}});a.warn("Old graph before copy",X(e)),A(i,e,u,i),e.setNode(i,{clusterNode:!0,id:i,clusterData:c.get(i).clusterData,label:c.get(i).label,graph:u}),a.warn("New graph after copy node: (",i,")",X(u)),a.debug("Old graph after copy",X(e))}else a.warn("Cluster ** ",i," **not meeting the criteria !externalConnections:",!c.get(i).externalConnections," no parent: ",!e.parent(i)," children ",e.children(i)&&e.children(i).length>0,e.children("D"),r),a.debug(c);n=e.nodes(),a.warn("New list of nodes",n);for(let i of n){let s=e.node(i);a.warn(" Now next level",i,s),s!=null&&s.clusterNode&&T(s.graph,r+1)}},"extractor"),L=w((e,r)=>{if(r.length===0)return[];let n=Object.assign([],r);return r.forEach(d=>{let l=e.children(d),o=L(e,l);n=[...n,...o]}),n},"sorter"),ie=w(e=>L(e,e.children()),"sortNodesByHierarchy"),F=w(async(e,r,n,d,l,o)=>{a.warn("Graph in recursive render:XAX",X(r),l);let i=r.graph().rankdir;a.trace("Dir in recursive render - dir:",i);let s=e.insert("g").attr("class","root");r.nodes()?a.info("Recursive render XXX",r.nodes()):a.info("No nodes found for",r),r.edges().length>0&&a.info("Recursive edges",r.edge(r.edges()[0]));let u=s.insert("g").attr("class","clusters"),b=s.insert("g").attr("class","edgePaths"),N=s.insert("g").attr("class","edgeLabels"),h=s.insert("g").attr("class","nodes");await Promise.all(r.nodes().map(async function(g){let t=r.node(g);if(l!==void 0){let f=JSON.parse(JSON.stringify(l.clusterData));a.trace(`Setting data for parent cluster XXX
 Node.id = `,g,`
 data=`,f.height,`
Parent cluster`,l.height),r.setNode(l.id,f),r.parent(g)||(a.trace("Setting parent",g,l.id),r.setParent(g,l.id,f))}if(a.info("(Insert) Node XXX"+g+": "+JSON.stringify(r.node(g))),t==null?void 0:t.clusterNode){a.info("Cluster identified XBX",g,t.width,r.node(g));let{ranksep:f,nodesep:v}=r.graph();t.graph.setGraph({...t.graph.graph(),ranksep:f+25,nodesep:v});let m=await F(h,t.graph,n,d,r.node(g),o),C=m.elem;M(t,C),t.diff=m.diff||0,a.info("New compound node after recursive render XAX",g,"width",t.width,"height",t.height),j(C,t)}else r.children(g).length>0?(a.trace("Cluster - the non recursive path XBX",g,t.id,t,t.width,"Graph:",r),a.trace(S(t.id,r)),c.set(t.id,{id:S(t.id,r),node:t})):(a.trace("Node - the non recursive path XAX",g,h,r.node(g),i),await Y(h,r.node(g),{config:o,dir:i}))})),await w(async()=>{let g=r.edges().map(async function(t){let f=r.edge(t.v,t.w,t.name);a.info("Edge "+t.v+" -> "+t.w+": "+JSON.stringify(t)),a.info("Edge "+t.v+" -> "+t.w+": ",t," ",JSON.stringify(r.edge(t))),a.info("Fix",c,"ids:",t.v,t.w,"Translating: ",c.get(t.v),c.get(t.w)),await H(N,f)});await Promise.all(g)},"processEdges")(),a.info("Graph before layout:",JSON.stringify(X(r))),a.info("############################################# XXX"),a.info("###                Layout                 ### XXX"),a.info("############################################# XXX"),V(r),a.info("Graph after layout:",JSON.stringify(X(r)));let p=0,{subGraphTitleTotalMargin:y}=W(o);return await Promise.all(ie(r).map(async function(g){var f;let t=r.node(g);if(a.info("Position XBX => "+g+": ("+t.x,","+t.y,") width: ",t.width," height: ",t.height),t==null?void 0:t.clusterNode)t.y+=y,a.info("A tainted cluster node XBX1",g,t.id,t.width,t.height,t.x,t.y,r.parent(g)),c.get(t.id).node=t,D(t);else if(r.children(g).length>0){a.info("A pure cluster node XBX1",g,t.id,t.x,t.y,t.width,t.height,r.parent(g)),t.height+=y,r.node(t.parentId);let v=(t==null?void 0:t.padding)/2||0,m=((f=t==null?void 0:t.labelBBox)==null?void 0:f.height)||0,C=m-v||0;a.debug("OffsetY",C,"labelHeight",m,"halfPadding",v),await $(u,t),c.get(t.id).node=t}else{let v=r.node(t.parentId);t.y+=y/2,a.info("A regular node XBX1 - using the padding",t.id,"parent",t.parentId,t.width,t.height,t.x,t.y,"offsetY",t.offsetY,"parent",v,v==null?void 0:v.offsetY,t),D(t)}})),r.edges().forEach(function(g){let t=r.edge(g);a.info("Edge "+g.v+" -> "+g.w+": "+JSON.stringify(t),t),t.points.forEach(C=>C.y+=y/2);let f=r.node(g.v);var v=r.node(g.w);let m=_(b,t,c,n,f,v,d);q(t,m)}),r.nodes().forEach(function(g){let t=r.node(g);a.info(g,t.type,t.diff),t.isGroup&&(p=t.diff)}),a.warn("Returning from recursive render XAX",s,p),{elem:s,diff:p}},"recursiveRender"),ce=w(async(e,r)=>{var o,i,s,u,b,N;let n=new J({multigraph:!0,compound:!0}).setGraph({rankdir:e.direction,nodesep:((o=e.config)==null?void 0:o.nodeSpacing)||((s=(i=e.config)==null?void 0:i.flowchart)==null?void 0:s.nodeSpacing)||e.nodeSpacing,ranksep:((u=e.config)==null?void 0:u.rankSpacing)||((N=(b=e.config)==null?void 0:b.flowchart)==null?void 0:N.rankSpacing)||e.rankSpacing,marginx:8,marginy:8}).setDefaultEdgeLabel(function(){return{}}),d=r.select("g");Q(d,e.markers,e.type,e.diagramId),Z(),z(),K(),ne(),e.nodes.forEach(h=>{n.setNode(h.id,{...h}),h.parentId&&n.setParent(h.id,h.parentId)}),a.debug("Edges:",e.edges),e.edges.forEach(h=>{if(h.start===h.end){let p=h.start,y=p+"---"+p+"---1",g=p+"---"+p+"---2",t=n.node(p);n.setNode(y,{domId:y,id:y,parentId:t.parentId,labelStyle:"",label:"",padding:0,shape:"labelRect",style:"",width:10,height:10}),n.setParent(y,t.parentId),n.setNode(g,{domId:g,id:g,parentId:t.parentId,labelStyle:"",padding:0,shape:"labelRect",label:"",style:"",width:10,height:10}),n.setParent(g,t.parentId);let f=structuredClone(h),v=structuredClone(h),m=structuredClone(h);f.label="",f.arrowTypeEnd="none",f.id=p+"-cyclic-special-1",v.arrowTypeEnd="none",v.id=p+"-cyclic-special-mid",m.label="",t.isGroup&&(f.fromCluster=p,m.toCluster=p),m.id=p+"-cyclic-special-2",n.setEdge(p,y,f,p+"-cyclic-special-0"),n.setEdge(y,g,v,p+"-cyclic-special-1"),n.setEdge(g,p,m,p+"-cyc<lic-special-2")}else n.setEdge(h.start,h.end,{...h},h.id)}),a.warn("Graph at first:",JSON.stringify(X(n))),ae(n),a.warn("Graph after XAX:",JSON.stringify(X(n)));let l=U();await F(d,n,e.type,e.diagramId,void 0,l)},"render");export{ce as render};
