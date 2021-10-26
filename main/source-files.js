var N = null;var sourcesIndex = {};
sourcesIndex["bonsaidb"] = {"name":"","dirs":[{"name":"cli","files":["admin.rs"]}],"files":["cli.rs","lib.rs"]};
sourcesIndex["bonsaidb_client"] = {"name":"","dirs":[{"name":"client","dirs":[{"name":"remote_database","files":["kv.rs","pubsub.rs"]}],"files":["quic_worker.rs","remote_database.rs","tungstenite_worker.rs"]}],"files":["builder.rs","client.rs","error.rs","lib.rs"]};
sourcesIndex["bonsaidb_core"] = {"name":"","dirs":[{"name":"admin","files":["database.rs","group.rs","mod.rs","password_config.rs","role.rs","user.rs"]},{"name":"document","files":["revision.rs"]},{"name":"kv","dirs":[{"name":"implementation","files":["get.rs","increment.rs","namespaced.rs","set.rs"]}],"files":["timestamp.rs"]},{"name":"permissions","files":["bonsai.rs","mod.rs"]},{"name":"schema","dirs":[{"name":"view","files":["map.rs"]}],"files":["collection.rs","mod.rs","names.rs","schematic.rs","view.rs"]}],"files":["connection.rs","custom_api.rs","document.rs","kv.rs","lib.rs","limits.rs","networking.rs","pubsub.rs","test_util.rs","transaction.rs"]};
sourcesIndex["bonsaidb_local"] = {"name":"","dirs":[{"name":"database","files":["kv.rs","pubsub.rs"]},{"name":"jobs","dirs":[{"name":"manager","files":["jobs.rs","managed_job.rs"]}],"files":["manager.rs","mod.rs","task.rs","traits.rs"]},{"name":"views","files":["integrity_scanner.rs","mapper.rs"]}],"files":["config.rs","database.rs","error.rs","lib.rs","open_trees.rs","storage.rs","tasks.rs","vault.rs","views.rs"]};
sourcesIndex["bonsaidb_macros"] = {"name":"","files":["lib.rs"]};
sourcesIndex["bonsaidb_server"] = {"name":"","dirs":[{"name":"cli","files":["certificate.rs","mod.rs","serve.rs"]},{"name":"server","files":["connected_client.rs","database.rs"]}],"files":["async_io_util.rs","backend.rs","config.rs","error.rs","lib.rs","server.rs","test_util.rs"]};
sourcesIndex["circulate"] = {"name":"","files":["lib.rs"]};
sourcesIndex["xtask"] = {"name":"","files":["main.rs"]};
createSourceSidebar();
