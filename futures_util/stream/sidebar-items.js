initSidebarItems({"fn":[["abortable","Creates a new `Abortable` stream and an `AbortHandle` which can be used to stop it."],["empty","Creates a stream which contains no elements."],["iter","Converts an `Iterator` into a `Stream` which is always ready to yield the next value."],["once","Creates a stream of a single element."],["pending","Creates a stream which never returns any elements."],["poll_fn","Creates a new stream wrapping a function returning `Poll<Option<T>>`."],["repeat","Create a stream which produces the same item repeatedly."],["repeat_with","Creates a new stream that repeats elements of type `A` endlessly by applying the provided closure, the repeater, `F: FnMut() -> A`."],["select","This function will attempt to pull items from both streams. Each stream will be polled in a round-robin fashion, and whenever a stream is ready to yield an item that item is yielded."],["try_unfold","Creates a `TryStream` from a seed and a closure returning a `TryFuture`."],["unfold","Creates a `Stream` from a seed and a closure returning a `Future`."]],"mod":[["futures_unordered","An unbounded set of futures."],["select_all","An unbounded set of streams"]],"struct":[["AbortHandle","A handle to an `Abortable` task."],["AbortRegistration","A registration handle for an `Abortable` task. Values of this type can be acquired from `AbortHandle::new` and are used in calls to `Abortable::new`."],["Abortable","A future/stream which can be remotely short-circuited using an `AbortHandle`."],["Aborted","Indicator that the `Abortable` task was aborted."],["AndThen","Stream for the `and_then` method."],["BufferUnordered","Stream for the `buffer_unordered` method."],["Buffered","Stream for the `buffered` method."],["CatchUnwind","Stream for the `catch_unwind` method."],["Chain","Stream for the `chain` method."],["Chunks","Stream for the `chunks` method."],["Collect","Future for the `collect` method."],["Concat","Future for the `concat` method."],["Cycle","Stream for the `cycle` method."],["Empty","Stream for the [`empty`] function."],["Enumerate","Stream for the `enumerate` method."],["ErrInto","Stream for the `err_into` method."],["Filter","Stream for the `filter` method."],["FilterMap","Stream for the `filter_map` method."],["FlatMap","Stream for the `flat_map` method."],["Flatten","Stream for the `flatten` method."],["Fold","Future for the `fold` method."],["ForEach","Future for the `for_each` method."],["ForEachConcurrent","Future for the `for_each_concurrent` method."],["Forward","Future for the `forward` method."],["Fuse","Stream for the `fuse` method."],["FuturesOrdered","An unbounded queue of futures."],["FuturesUnordered","A set of futures which may complete in any order."],["Inspect","Stream for the `inspect` method."],["InspectErr","Stream for the `inspect_err` method."],["InspectOk","Stream for the `inspect_ok` method."],["IntoStream","Stream for the `into_stream` method."],["Iter","Stream for the [`iter`] function."],["Map","Stream for the `map` method."],["MapErr","Stream for the `map_err` method."],["MapOk","Stream for the `map_ok` method."],["Next","Future for the `next` method."],["NextIf","Future for the `Peekable::next_if` method."],["NextIfEq","Future for the `Peekable::next_if_eq` method."],["Once","A stream which emits single element and then EOF."],["OrElse","Stream for the `or_else` method."],["Peek","Future for the `Peekable::peek` method."],["Peekable","A `Stream` that implements a `peek` method."],["Pending","Stream for the [`pending()`] function."],["PollFn","Stream for the [`poll_fn`] function."],["ReadyChunks","Stream for the `ready_chunks` method."],["Repeat","Stream for the [`repeat`] function."],["RepeatWith","An stream that repeats elements of type `A` endlessly by applying the provided closure `F: FnMut() -> A`."],["ReuniteError","Error indicating a `SplitSink<S>` and `SplitStream<S>` were not two halves of a `Stream + Split`, and thus could not be `reunite`d."],["Scan","Stream for the `scan` method."],["Select","Stream for the [`select()`] function."],["SelectNextSome","Future for the `select_next_some` method."],["Skip","Stream for the `skip` method."],["SkipWhile","Stream for the `skip_while` method."],["SplitSink","A `Sink` part of the split pair"],["SplitStream","A `Stream` part of the split pair"],["StreamFuture","Future for the `into_future` method."],["Take","Stream for the `take` method."],["TakeUntil","Stream for the `take_until` method."],["TakeWhile","Stream for the `take_while` method."],["Then","Stream for the `then` method."],["TryBufferUnordered","Stream for the `try_buffer_unordered` method."],["TryBuffered","Stream for the `try_buffered` method."],["TryCollect","Future for the `try_collect` method."],["TryConcat","Future for the `try_concat` method."],["TryFilter","Stream for the `try_filter` method."],["TryFilterMap","Stream for the `try_filter_map` method."],["TryFlatten","Stream for the `try_flatten` method."],["TryFold","Future for the `try_fold` method."],["TryForEach","Future for the `try_for_each` method."],["TryForEachConcurrent","Future for the `try_for_each_concurrent` method."],["TryNext","Future for the `try_next` method."],["TrySkipWhile","Stream for the `try_skip_while` method."],["TryTakeWhile","Stream for the `try_take_while` method."],["TryUnfold","Stream for the [`try_unfold`] function."],["Unfold","Stream for the [`unfold`] function."],["Unzip","Future for the `unzip` method."],["Zip","Stream for the `zip` method."]],"trait":[["FusedStream","A stream which tracks whether or not the underlying stream should no longer be polled."],["Stream","A stream of values produced asynchronously."],["StreamExt","An extension trait for `Stream`s that provides a variety of convenient combinator functions."],["TryStream","A convenience for streams that return `Result` values that includes a variety of adapters tailored to such futures."],["TryStreamExt","Adapters specific to `Result`-returning streams"]],"type":[["BoxStream","An owned dynamically typed [`Stream`] for use in cases where you can’t statically type your result or need to add some indirection."],["LocalBoxStream","`BoxStream`, but without the `Send` requirement."]]});