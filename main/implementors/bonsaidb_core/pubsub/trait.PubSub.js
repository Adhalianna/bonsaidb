(function() {var implementors = {};
implementors["bonsaidb_client"] = [{"text":"impl&lt;DB, A&gt; <a class=\"trait\" href=\"bonsaidb_core/pubsub/trait.PubSub.html\" title=\"trait bonsaidb_core::pubsub::PubSub\">PubSub</a> for <a class=\"struct\" href=\"bonsaidb_client/struct.RemoteDatabase.html\" title=\"struct bonsaidb_client::RemoteDatabase\">RemoteDatabase</a>&lt;DB, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;DB: <a class=\"trait\" href=\"bonsaidb_core/schema/trait.Schema.html\" title=\"trait bonsaidb_core::schema::Schema\">Schema</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"bonsaidb_core/custom_api/trait.CustomApi.html\" title=\"trait bonsaidb_core::custom_api::CustomApi\">CustomApi</a>,&nbsp;</span>","synthetic":false,"types":["bonsaidb_client::client::remote_database::RemoteDatabase"]}];
implementors["bonsaidb_core"] = [];
implementors["bonsaidb_server"] = [{"text":"impl&lt;'a, B:&nbsp;<a class=\"trait\" href=\"bonsaidb_server/trait.Backend.html\" title=\"trait bonsaidb_server::Backend\">Backend</a>, DB:&nbsp;<a class=\"trait\" href=\"bonsaidb_core/schema/trait.Schema.html\" title=\"trait bonsaidb_core::schema::Schema\">Schema</a>&gt; <a class=\"trait\" href=\"bonsaidb_core/pubsub/trait.PubSub.html\" title=\"trait bonsaidb_core::pubsub::PubSub\">PubSub</a> for <a class=\"struct\" href=\"bonsaidb_server/struct.ServerDatabase.html\" title=\"struct bonsaidb_server::ServerDatabase\">ServerDatabase</a>&lt;'a, B, DB&gt;","synthetic":false,"types":["bonsaidb_server::server::database::ServerDatabase"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()