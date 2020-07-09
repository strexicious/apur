use futures::{
    future::{self, BoxFuture},
    task::Context,
    Future,
};

/// Given a Future and a code which provokes this Future to progress,
/// return a Future which when polled polls the given Future
/// and calls the progressor code in preparation for next poll.
pub fn post_pending<'a, R: 'a>(
    mut future: BoxFuture<'a, R>,
    mut code: impl FnMut() + 'a,
) -> impl Future<Output = R> + 'a {
    future::poll_fn(move |cx: &mut Context<'_>| {
        let poll_res = future.as_mut().poll(cx);

        if poll_res.is_pending() {
            code()
        }

        poll_res
    })
}
