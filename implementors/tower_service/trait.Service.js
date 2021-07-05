(function() {var implementors = {};
implementors["hyper_tls"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"http/uri/struct.Uri.html\" title=\"struct http::uri::Uri\">Uri</a>&gt; for <a class=\"struct\" href=\"hyper_tls/struct.HttpsConnector.html\" title=\"struct hyper_tls::HttpsConnector\">HttpsConnector</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;<a class=\"struct\" href=\"http/uri/struct.Uri.html\" title=\"struct http::uri::Uri\">Uri</a>&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"tower_service/trait.Service.html#associatedtype.Response\" title=\"type tower_service::Service::Response\">Response</a>: <a class=\"trait\" href=\"tokio/io/async_read/trait.AsyncRead.html\" title=\"trait tokio::io::async_read::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"tokio/io/async_write/trait.AsyncWrite.html\" title=\"trait tokio::io::async_write::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"tower_service/trait.Service.html#associatedtype.Future\" title=\"type tower_service::Service::Future\">Future</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"tower_service/trait.Service.html#associatedtype.Error\" title=\"type tower_service::Service::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html\" title=\"struct alloc::boxed::Box\">Box</a>&lt;dyn <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["hyper_tls::client::HttpsConnector"]}];
implementors["routerify"] = [{"text":"impl&lt;B:&nbsp;<a class=\"trait\" href=\"http_body/trait.Body.html\" title=\"trait http_body::Body\">HttpBody</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static, E:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html\" title=\"struct alloc::boxed::Box\">Box</a>&lt;dyn <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/error/trait.Error.html\" title=\"trait std::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>&gt;&gt; + 'static&gt; <a class=\"trait\" href=\"tower_service/trait.Service.html\" title=\"trait tower_service::Service\">Service</a>&lt;&amp;'_ <a class=\"struct\" href=\"hyper/server/tcp/addr_stream/struct.AddrStream.html\" title=\"struct hyper::server::tcp::addr_stream::AddrStream\">AddrStream</a>&gt; for <a class=\"struct\" href=\"routerify/struct.RouterService.html\" title=\"struct routerify::RouterService\">RouterService</a>&lt;B, E&gt;","synthetic":false,"types":["routerify::service::router_service::RouterService"]}];
implementors["tower_service"] = [];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()