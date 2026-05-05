Manually fix some structs
Comment out some apis for now
Fix bool decoding

Replace in `lib/mattermost-api/src/models`
```
^(\s+?#[^\r\n]+?)\)\](\r?\n\s+?[^\r\n]+?Option<bool>,\r?\n)
```
```
$1, default, deserialize_with = "bool_parser::deserialize_option_bool")]$2
```

Manual compatibility patch (Mattermost can return embed type `permalink`):

```diff
diff --git a/lib/mattermost-api/src/models/post_metadata_embeds_inner.rs b/lib/mattermost-api/src/models/post_metadata_embeds_inner.rs
@@
 pub enum Type {
     #[serde(rename = "image")]
     Image,
     #[serde(rename = "message_attachment")]
     MessageAttachment,
     #[serde(rename = "opengraph")]
     Opengraph,
     #[serde(rename = "link")]
     Link,
+    #[serde(rename = "permalink")]
+    Permalink,
+    #[serde(other)]
+    Unknown,
 }
```
