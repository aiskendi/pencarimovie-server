(function defineNuvioEnv() {
  var root = typeof globalThis !== "undefined" ? globalThis : window;
  var env = root.__NUVIO_ENV__ || {};
  var values = {
  "NUVIO_SUPABASE_URL": "",
  "NUVIO_SUPABASE_ANON_KEY": "",
  "NUVIO_SUPABASE_FALLBACK_URL": "",
  "TV_LOGIN_WEB_BASE_URL": "https://nuvio.tv/tv-login",
  "DEVICE_LOGIN_WEB_BASE_URL": "https://nuvio.tv/link",
  "YOUTUBE_PROXY_URL": "youtube-proxy.html",
  "INTRODB_API_URL": "https://api.introdb.app/",
  "IMDB_RATINGS_API_BASE_URL": "",
  "IMDB_TAPFRAME_API_BASE_URL": "",
  "AVATAR_PUBLIC_BASE_URL": "",
  "UNIQUE_CONTRIBUTIONS_BASE_URL": "",
  "SUPPORTERS_API_BASE_URL": "https://nuvio.tv/",
  "SUPPORT_URL": "https://nuvio.tv/support",
  "SPONSOR_NAMES": "ragmehos.",
  "TMDB_API_KEY": "",
  "TRAKT_CLIENT_ID": "",
  "TRAKT_CLIENT_SECRET": "",
  "SIMKL_CLIENT_ID": "",
  "SIMKL_APP_NAME": "nuvio",
  "PREMIUMIZE_CLIENT_ID": ""
};
  for (var key in values) {
    if (Object.prototype.hasOwnProperty.call(values, key)) {
      env[key] = values[key];
    }
  }
  root.__NUVIO_ENV__ = env;
}());
