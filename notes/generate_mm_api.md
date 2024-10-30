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

