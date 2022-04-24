(function() {var implementors = {};
implementors["bonsaidb_core"] = [{"text":"impl&lt;K:&nbsp;for&lt;'a&gt; <a class=\"trait\" href=\"bonsaidb_core/key/trait.Key.html\" title=\"trait bonsaidb_core::key::Key\">Key</a>&lt;'a&gt;, V&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"enum\" href=\"bonsaidb_core/schema/view/map/enum.MappingsIter.html\" title=\"enum bonsaidb_core::schema::view::map::MappingsIter\">MappingsIter</a>&lt;K, V&gt;","synthetic":false,"types":["bonsaidb_core::schema::view::map::MappingsIter"]},{"text":"impl&lt;'a, D, V:&nbsp;<a class=\"trait\" href=\"bonsaidb_core/schema/view/trait.View.html\" title=\"trait bonsaidb_core::schema::view::View\">View</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"bonsaidb_core/schema/view/map/struct.MappedDocumentsIter.html\" title=\"struct bonsaidb_core::schema::view::map::MappedDocumentsIter\">MappedDocumentsIter</a>&lt;'a, D, V&gt;","synthetic":false,"types":["bonsaidb_core::schema::view::map::MappedDocumentsIter"]},{"text":"impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"bonsaidb_core/transaction/struct.DocumentChangesIter.html\" title=\"struct bonsaidb_core::transaction::DocumentChangesIter\">DocumentChangesIter</a>&lt;'a&gt;","synthetic":false,"types":["bonsaidb_core::transaction::DocumentChangesIter"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"bonsaidb_core/transaction/struct.DocumentChangesIntoIter.html\" title=\"struct bonsaidb_core::transaction::DocumentChangesIntoIter\">DocumentChangesIntoIter</a>","synthetic":false,"types":["bonsaidb_core::transaction::DocumentChangesIntoIter"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"bonsaidb_core/pubsub/struct.Receiver.html\" title=\"struct bonsaidb_core::pubsub::Receiver\">Receiver</a>","synthetic":false,"types":["bonsaidb_core::pubsub::Receiver"]}];
implementors["bonsaidb_files"] = [{"text":"impl&lt;'a, Database:&nbsp;<a class=\"trait\" href=\"bonsaidb_core/connection/trait.Connection.html\" title=\"trait bonsaidb_core::connection::Connection\">Connection</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>, Config:&nbsp;<a class=\"trait\" href=\"bonsaidb_files/trait.FileConfig.html\" title=\"trait bonsaidb_files::FileConfig\">FileConfig</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.60.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"bonsaidb_files/direct/struct.Contents.html\" title=\"struct bonsaidb_files::direct::Contents\">Contents</a>&lt;'a, <a class=\"struct\" href=\"bonsaidb_files/direct/struct.Blocking.html\" title=\"struct bonsaidb_files::direct::Blocking\">Blocking</a>&lt;Database&gt;, Config&gt;","synthetic":false,"types":["bonsaidb_files::direct::Contents"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()