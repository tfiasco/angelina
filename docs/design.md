# Angelina: Build Graph Database Atop Key Value Store



## DataModel

### Schema

```
| SchemaType | SchemaID | SchemaProperties |
| -------- KEY -------- | ---- VALUE ----- |
```

- Key
  - `SchemaType` can be one of `VertexLabel, EdgeLabel, PropertyKey`

### Vertex

```
| KeyType | VertexID | VertexLabel | Property 1 | Property 2 | Property 3 | ... |
| ------ KEY ------- | ----------------------- VALUE -------------------------- |
```

- Key
  - `KeyType` : `Vertex`

### Edge



```
OutEdge:
 | KeyType | SrcVertexID | EdgeLabel | DstVertexID | EdgeID | Property 1 | Property 2 | ... |
 | -------------------------- KEY ------------------------- | --------- VALUE ------------- |
InEdge:
 | KeyType | DstVertexID | EdgeLabel | SrcVertexID | EdgeID | Property 1 | Property 2 | ... |
 | -------------------------- KEY ------------------------- | --------- VALUE ------------- |
```

- Key
  - `KeyType` : `Edge`

### Property

Vertex 和 Edge 的 Property 的结构是一样的。

```python
| property 1 (id, len, eid, value) | property 2 (id, len, eid, value) | property 3 (id, len, eid, value) | ... |
```

- 每一格是一个 `Tuple (Long, Long, Long, Value)` ，表示属性的 id ，长度，`eid` (short for `element id`)，属性的值。

  

### MetaProperty

```
| KeyType | PropertyID | PropertyEID | FullKey | Property 1 | Property 2 | Property 3 | ... | 
| -------------------- KEY ------------------- | ---------------- VALUE ------------------- |
```

- Key
  - 第一个 KeyType 可以是 `VertexMetaProperty, EdgeMetaProperty, MetaPropertyMetaProperty`
  - `FullKey` 是 `Vertex, Edge, MetaProperty` 的完整的 `Key`



## Angelina SQL

### DDL

```SQL
CREATE GRAPH graph_name;

CREATE VERTEX LABEL (label_name);

CREATE EDGE LABEL (label_name, multiplicity);

CREATE PROPERTY KEY (key_name, cardinality);
```



```SQL
DROP GRAPH graph_name;

DROP VERTEX LABEL (label_name);

DROP EDGE LABEL (label_name);

DROP PROPERTY KEY (key_name);
```



### DML

```SQL
INSERT VERTEX "label_name" PROPERTIES (prop1, prop2) VALUES ("vertex_id"):("value1", "value2");

INSERT EDGE "label_name" PROPERTIES (prop1, prop2) 
	VALUES ("vertex_id_1" -> "vertex_id_2"):("value1", "value2");
```



```SQL
UPDATE v.prop1 = 1, v.prop2 = 2, delete v3.prop1, e.prop3 = 3, e2.prop4 = 4
	FROM (v) - [e] -> (v2) - [e2] -> (v3), (v4) - [..*] -> (v5)
	WHERE v.label == "label1" AND e.label == "edge1" AND v2.label == "label3"
		  AND v.xx > 6 AND func(v2.p1) < 3 AND v3.p0 == v4.p0 AND v5.p0 IS NOT NULL;
```



### Query

```SQL
SELECT v.p1, v.p2, e.label, e.p1, v2.p3, v2.p4, v5.*
	FROM (v) - [e] -> (v2) - [e2] -> (v3), (v4) - [*] -> (v5)
	WHERE v.label == "label1" AND e.label == "edge1" AND v2.label == "label3"
		AND v.xx > 6 AND func(v2.p1) < 3 AND v3.p0 == v4.p0 AND v5.p0 IS NOT NULL
	ORDER BY v1.prop3
	LIMIT n
```



### Pipeline Update

```SQL
SELECT * 
	FROM (v) - [e] -> (v2) - [e2] -> (v3)
	WHERE v.label == "label1" AND e.label == "edge1" AND v2.label == "label3"
		AND v.xx > 6 AND func(v2.p1) < 3  | 
UPDATE v.prop1 = 3, v2.prop2 = 4, e.prop3 = 5  |
INSERT EDGE "label_name" PROPERTIES (prop1, prop2) VALUES (v -> v3):("value1", "value2")  |
DELETE e2
```



