(function() {var implementors = {};
implementors["bonsaidb_core"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/connection/struct.Database.html\" title=\"struct bonsaidb_core::connection::Database\">Database</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/connection/struct.Database.html\" title=\"struct bonsaidb_core::connection::Database\">Database</a>","synthetic":false,"types":["bonsaidb_core::connection::Database"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/document/struct.Revision.html\" title=\"struct bonsaidb_core::document::Revision\">Revision</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/document/struct.Revision.html\" title=\"struct bonsaidb_core::document::Revision\">Revision</a>","synthetic":false,"types":["bonsaidb_core::document::revision::Revision"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/document/struct.Header.html\" title=\"struct bonsaidb_core::document::Header\">Header</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/document/struct.Header.html\" title=\"struct bonsaidb_core::document::Header\">Header</a>","synthetic":false,"types":["bonsaidb_core::document::Header"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"bonsaidb_core/document/enum.KeyId.html\" title=\"enum bonsaidb_core::document::KeyId\">KeyId</a>&gt; for <a class=\"enum\" href=\"bonsaidb_core/document/enum.KeyId.html\" title=\"enum bonsaidb_core::document::KeyId\">KeyId</a>","synthetic":false,"types":["bonsaidb_core::document::KeyId"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"bonsaidb_core/schema/enum.NamedReference.html\" title=\"enum bonsaidb_core::schema::NamedReference\">NamedReference</a>&lt;'a&gt;&gt; for <a class=\"enum\" href=\"bonsaidb_core/schema/enum.NamedReference.html\" title=\"enum bonsaidb_core::schema::NamedReference\">NamedReference</a>&lt;'a&gt;","synthetic":false,"types":["bonsaidb_core::schema::collection::NamedReference"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/struct.Name.html\" title=\"struct bonsaidb_core::schema::Name\">Name</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/struct.Name.html\" title=\"struct bonsaidb_core::schema::Name\">Name</a>","synthetic":false,"types":["bonsaidb_core::schema::names::Name"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/struct.Authority.html\" title=\"struct bonsaidb_core::schema::Authority\">Authority</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/struct.Authority.html\" title=\"struct bonsaidb_core::schema::Authority\">Authority</a>","synthetic":false,"types":["bonsaidb_core::schema::names::Authority"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/struct.SchemaName.html\" title=\"struct bonsaidb_core::schema::SchemaName\">SchemaName</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/struct.SchemaName.html\" title=\"struct bonsaidb_core::schema::SchemaName\">SchemaName</a>","synthetic":false,"types":["bonsaidb_core::schema::names::SchemaName"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/struct.CollectionName.html\" title=\"struct bonsaidb_core::schema::CollectionName\">CollectionName</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/struct.CollectionName.html\" title=\"struct bonsaidb_core::schema::CollectionName\">CollectionName</a>","synthetic":false,"types":["bonsaidb_core::schema::names::CollectionName"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/struct.ViewName.html\" title=\"struct bonsaidb_core::schema::ViewName\">ViewName</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/struct.ViewName.html\" title=\"struct bonsaidb_core::schema::ViewName\">ViewName</a>","synthetic":false,"types":["bonsaidb_core::schema::names::ViewName"]},{"text":"impl&lt;K:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"bonsaidb_core/schema/view/map/trait.Key.html\" title=\"trait bonsaidb_core::schema::view::map::Key\">Key</a>, V:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://docs.rs/serde/1.0.130/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/view/map/struct.Map.html\" title=\"struct bonsaidb_core::schema::view::map::Map\">Map</a>&lt;K, V&gt;&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/view/map/struct.Map.html\" title=\"struct bonsaidb_core::schema::view::map::Map\">Map</a>&lt;K, V&gt;","synthetic":false,"types":["bonsaidb_core::schema::view::map::Map"]},{"text":"impl&lt;K:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"bonsaidb_core/schema/view/map/trait.Key.html\" title=\"trait bonsaidb_core::schema::view::map::Key\">Key</a>, V:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/schema/view/map/struct.MappedValue.html\" title=\"struct bonsaidb_core::schema::view::map::MappedValue\">MappedValue</a>&lt;K, V&gt;&gt; for <a class=\"struct\" href=\"bonsaidb_core/schema/view/map/struct.MappedValue.html\" title=\"struct bonsaidb_core::schema::view::map::MappedValue\">MappedValue</a>&lt;K, V&gt;","synthetic":false,"types":["bonsaidb_core::schema::view::map::MappedValue"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/kv/struct.Timestamp.html\" title=\"struct bonsaidb_core::kv::Timestamp\">Timestamp</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/kv/struct.Timestamp.html\" title=\"struct bonsaidb_core::kv::Timestamp\">Timestamp</a>","synthetic":false,"types":["bonsaidb_core::kv::timestamp::Timestamp"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"bonsaidb_core/kv/enum.Value.html\" title=\"enum bonsaidb_core::kv::Value\">Value</a>&gt; for <a class=\"enum\" href=\"bonsaidb_core/kv/enum.Value.html\" title=\"enum bonsaidb_core::kv::Value\">Value</a>","synthetic":false,"types":["bonsaidb_core::kv::Value"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"bonsaidb_core/kv/enum.Numeric.html\" title=\"enum bonsaidb_core::kv::Numeric\">Numeric</a>&gt; for <a class=\"enum\" href=\"bonsaidb_core/kv/enum.Numeric.html\" title=\"enum bonsaidb_core::kv::Numeric\">Numeric</a>","synthetic":false,"types":["bonsaidb_core::kv::Numeric"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"bonsaidb_core/kv/enum.KeyStatus.html\" title=\"enum bonsaidb_core::kv::KeyStatus\">KeyStatus</a>&gt; for <a class=\"enum\" href=\"bonsaidb_core/kv/enum.KeyStatus.html\" title=\"enum bonsaidb_core::kv::KeyStatus\">KeyStatus</a>","synthetic":false,"types":["bonsaidb_core::kv::KeyStatus"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/test_util/struct.Basic.html\" title=\"struct bonsaidb_core::test_util::Basic\">Basic</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/test_util/struct.Basic.html\" title=\"struct bonsaidb_core::test_util::Basic\">Basic</a>","synthetic":false,"types":["bonsaidb_core::test_util::Basic"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_core/test_util/struct.Unique.html\" title=\"struct bonsaidb_core::test_util::Unique\">Unique</a>&gt; for <a class=\"struct\" href=\"bonsaidb_core/test_util/struct.Unique.html\" title=\"struct bonsaidb_core::test_util::Unique\">Unique</a>","synthetic":false,"types":["bonsaidb_core::test_util::Unique"]}];
implementors["bonsaidb_jobs"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"struct\" href=\"bonsaidb_jobs/task/struct.Id.html\" title=\"struct bonsaidb_jobs::task::Id\">Id</a>&gt; for <a class=\"struct\" href=\"bonsaidb_jobs/task/struct.Id.html\" title=\"struct bonsaidb_jobs::task::Id\">Id</a>","synthetic":false,"types":["bonsaidb_jobs::task::Id"]}];
implementors["bonsaidb_server"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.54.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>&lt;<a class=\"enum\" href=\"bonsaidb_server/enum.Transport.html\" title=\"enum bonsaidb_server::Transport\">Transport</a>&gt; for <a class=\"enum\" href=\"bonsaidb_server/enum.Transport.html\" title=\"enum bonsaidb_server::Transport\">Transport</a>","synthetic":false,"types":["bonsaidb_server::server::connected_client::Transport"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()