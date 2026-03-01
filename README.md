# Bunpro Notifier

A simple app that sends you system notifications when it's time to do your reviews.

You may use the 3 buttons (windows only) on the toast to:

- Grammar: Go straight into doing your grammar reviews
- Vocab: Go straight into doing your vocab reviews
- Dashboard: Go to your bunpro dashboard

![Screenshot of Notification](https://raw.githubusercontent.com/MolotovCherry/BunproNotifier/refs/heads/main/_doc/notification.png)

## Refresh data

By default, new data is pulled from the api every hour, but you can change this in the settings. If for any reason you need to refresh immediately, you can find the bunpro icon in the systray, right click on it, and click "Refresh Data".

## Closing app

Find the bunpro icon in the systray, right click on it, and click "Quit".

## Configuration

The configuration mirrors the options on Bunpro's forecast section on the homepage.

Note with regards to the api token, this is _not_ the api token found in your settings. The token needs to be found by observing the api request authorization header. It'll appear similar to `authorization: Token token=<token>`

```ron
(
    account: (
        api_token: "<token_here>",
    ),
    forecast: (
        // 1-65536 : How many minutes between each data update.
        // Grabs updated information from bunpro api; you'll want this
        // at quicker rates if actively doing reviews and interval is set
        // to hourly, as the program's cached info could get stale.
        //
        // if interval is daily, interval should be set much higher, as
        // there's no need to refresh every minute
        update_rate: 1,

        // Hourly|Daily(hour: 0-23): Notify for reviews hourly or daily
        interval: Hourly, // or Daily( hour: 6 )

        // TotalReviews|NewOnly : Show total review count or new only count
        count: NewOnly,

        // false|true : Notify for new grammar reviews
        grammar: true,

        // false|true : Notify for new vocab reviews
        vocab: true,

        // false|true : Notify what reviews are available on startup
        initial_notify: true
    ),
)
```

# Disclaimer

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
