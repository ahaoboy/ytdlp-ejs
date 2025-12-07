// Browser environment simulation for YouTube player execution
if (typeof globalThis.XMLHttpRequest === "undefined") {
  globalThis.XMLHttpRequest = { prototype: {} };
}
if (typeof globalThis.window === "undefined") {
  globalThis.window = Object.create(null);
}
if (typeof globalThis.g === "undefined") {
  globalThis.g = Object.create(null);
}
if (typeof globalThis.URL === "undefined") {
  window.location = {
    hash: "",
    host: "www.youtube.com",
    hostname: "www.youtube.com",
    href: "https://www.youtube.com/watch?v=yt-dlp-wins",
    origin: "https://www.youtube.com",
    password: "",
    pathname: "/watch",
    port: "",
    protocol: "https:",
    search: "?v=yt-dlp-wins",
    username: "",
  };
} else {
  window.location = new URL("https://www.youtube.com/watch?v=yt-dlp-wins");
}
if (typeof globalThis.document === "undefined") {
  globalThis.document = Object.create(null);
}
if (typeof globalThis.navigator === "undefined") {
  globalThis.navigator = Object.create(null);
}
if (typeof globalThis.self === "undefined") {
  globalThis.self = globalThis;
}
