# BunproNotifier

A simple app that sends you system notifications when it's time to do your reviews.

## Configuration

The configuration mirrors the options on Bunpro's forecast section on the homepage.

Note with regards to the api token, this is _not_ the api token found in your settings. The token needs to be found by observing the api request authorization header. It'll appear similar to `authorization: Token token=<token>`

```ron
(
    account: (
        api_token: "<token_here>",
    ),
    forecast: (
        // Hourly|Daily : Notify for reviews hourly or daily (every 24 hours)
        interval: Hourly,
        // TotalReviews|NewOnly : Show total review count or new only count
        count: NewOnly,
        // 0-23 : 24 hour to send notification for Daily interval
        // defaults to 6 if invalid
        daily_time: 6,
        // 1-65536 : How many hours between each poll (updating information from online)
        poll_rate: 1,
        // false|true : Notify for new grammar reviews
        grammar: true,
        // false|true : Notify for new vocab reviews
        vocab: true,
        // false|true : Notify what reviews are available on startup
        initial_notify: true
    ),
)
```
