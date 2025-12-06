// Intl Polyfill - Minimal implementation for QuickJS compatibility
// Based on: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl

if (typeof globalThis.Intl === "undefined") {
    globalThis.Intl = {};
}

// Intl.Collator - String comparison
if (typeof Intl.Collator === "undefined") {
    Intl.Collator = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.Collator.prototype.compare = function(a, b) {
        if (a < b) return -1;
        if (a > b) return 1;
        return 0;
    };
    Intl.Collator.prototype.resolvedOptions = function() {
        return {
            locale: "en",
            usage: "sort",
            sensitivity: "variant",
            ignorePunctuation: false,
            collation: "default",
            numeric: false,
            caseFirst: "false"
        };
    };
    Intl.Collator.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.DateTimeFormat - Date/time formatting
if (typeof Intl.DateTimeFormat === "undefined") {
    Intl.DateTimeFormat = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.DateTimeFormat.prototype.format = function(date) {
        if (!(date instanceof Date)) date = new Date(date);
        return date.toString();
    };
    Intl.DateTimeFormat.prototype.formatToParts = function(date) {
        if (!(date instanceof Date)) date = new Date(date);
        return [{ type: "literal", value: date.toString() }];
    };
    Intl.DateTimeFormat.prototype.formatRange = function(startDate, endDate) {
        return this.format(startDate) + " – " + this.format(endDate);
    };
    Intl.DateTimeFormat.prototype.formatRangeToParts = function(startDate, endDate) {
        return [{ type: "literal", value: this.formatRange(startDate, endDate), source: "shared" }];
    };
    Intl.DateTimeFormat.prototype.resolvedOptions = function() {
        return {
            locale: "en",
            calendar: "gregory",
            numberingSystem: "latn",
            timeZone: "UTC"
        };
    };
    Intl.DateTimeFormat.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.DisplayNames - Display names for languages, regions, scripts, currencies
if (typeof Intl.DisplayNames === "undefined") {
    Intl.DisplayNames = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.DisplayNames.prototype.of = function(code) {
        return code;
    };
    Intl.DisplayNames.prototype.resolvedOptions = function() {
        return {
            locale: "en",
            style: "long",
            type: this.options.type || "language",
            fallback: "code"
        };
    };
    Intl.DisplayNames.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.DurationFormat - Duration formatting
if (typeof Intl.DurationFormat === "undefined") {
    Intl.DurationFormat = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.DurationFormat.prototype.format = function(duration) {
        var parts = [];
        if (duration.hours) parts.push(duration.hours + "h");
        if (duration.minutes) parts.push(duration.minutes + "m");
        if (duration.seconds) parts.push(duration.seconds + "s");
        return parts.join(" ") || "0s";
    };
    Intl.DurationFormat.prototype.formatToParts = function(duration) {
        return [{ type: "literal", value: this.format(duration) }];
    };
    Intl.DurationFormat.prototype.resolvedOptions = function() {
        return { locale: "en", style: "long" };
    };
    Intl.DurationFormat.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.ListFormat - List formatting
if (typeof Intl.ListFormat === "undefined") {
    Intl.ListFormat = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.ListFormat.prototype.format = function(list) {
        if (!Array.isArray(list)) return String(list);
        if (list.length === 0) return "";
        if (list.length === 1) return String(list[0]);
        if (list.length === 2) return list[0] + " and " + list[1];
        return list.slice(0, -1).join(", ") + ", and " + list[list.length - 1];
    };
    Intl.ListFormat.prototype.formatToParts = function(list) {
        return [{ type: "literal", value: this.format(list) }];
    };
    Intl.ListFormat.prototype.resolvedOptions = function() {
        return { locale: "en", type: "conjunction", style: "long" };
    };
    Intl.ListFormat.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.Locale - Locale identifier
if (typeof Intl.Locale === "undefined") {
    Intl.Locale = function(tag, options) {
        this.baseName = tag;
        this.language = tag.split("-")[0];
        this.region = undefined;
        this.script = undefined;
        this.calendar = (options && options.calendar) || undefined;
        this.caseFirst = (options && options.caseFirst) || undefined;
        this.collation = (options && options.collation) || undefined;
        this.hourCycle = (options && options.hourCycle) || undefined;
        this.numberingSystem = (options && options.numberingSystem) || undefined;
        this.numeric = (options && options.numeric) || false;
    };
    Intl.Locale.prototype.maximize = function() { return this; };
    Intl.Locale.prototype.minimize = function() { return this; };
    Intl.Locale.prototype.toString = function() { return this.baseName; };
    Intl.Locale.prototype.getCalendars = function() { return ["gregory"]; };
    Intl.Locale.prototype.getCollations = function() { return ["default"]; };
    Intl.Locale.prototype.getHourCycles = function() { return ["h23"]; };
    Intl.Locale.prototype.getNumberingSystems = function() { return ["latn"]; };
    Intl.Locale.prototype.getTextInfo = function() { return { direction: "ltr" }; };
    Intl.Locale.prototype.getTimeZones = function() { return ["UTC"]; };
    Intl.Locale.prototype.getWeekInfo = function() { return { firstDay: 1, weekend: [6, 7], minimalDays: 1 }; };
}

// Intl.NumberFormat - Number formatting
if (typeof Intl.NumberFormat === "undefined") {
    Intl.NumberFormat = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.NumberFormat.prototype.format = function(number) {
        return String(number);
    };
    Intl.NumberFormat.prototype.formatToParts = function(number) {
        return [{ type: "integer", value: String(number) }];
    };
    Intl.NumberFormat.prototype.formatRange = function(start, end) {
        return this.format(start) + "–" + this.format(end);
    };
    Intl.NumberFormat.prototype.formatRangeToParts = function(start, end) {
        return [{ type: "literal", value: this.formatRange(start, end), source: "shared" }];
    };
    Intl.NumberFormat.prototype.resolvedOptions = function() {
        return {
            locale: "en",
            numberingSystem: "latn",
            style: "decimal",
            minimumIntegerDigits: 1,
            minimumFractionDigits: 0,
            maximumFractionDigits: 3,
            useGrouping: true
        };
    };
    Intl.NumberFormat.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.PluralRules - Plural rules
if (typeof Intl.PluralRules === "undefined") {
    Intl.PluralRules = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.PluralRules.prototype.select = function(number) {
        if (number === 1) return "one";
        return "other";
    };
    Intl.PluralRules.prototype.selectRange = function(start, end) {
        return "other";
    };
    Intl.PluralRules.prototype.resolvedOptions = function() {
        return {
            locale: "en",
            type: "cardinal",
            minimumIntegerDigits: 1,
            minimumFractionDigits: 0,
            maximumFractionDigits: 3,
            pluralCategories: ["one", "other"]
        };
    };
    Intl.PluralRules.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.RelativeTimeFormat - Relative time formatting
if (typeof Intl.RelativeTimeFormat === "undefined") {
    Intl.RelativeTimeFormat = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
    };
    Intl.RelativeTimeFormat.prototype.format = function(value, unit) {
        var absValue = Math.abs(value);
        var unitStr = absValue === 1 ? unit.replace(/s$/, "") : unit;
        if (value < 0) return absValue + " " + unitStr + " ago";
        if (value > 0) return "in " + absValue + " " + unitStr;
        return "now";
    };
    Intl.RelativeTimeFormat.prototype.formatToParts = function(value, unit) {
        return [{ type: "literal", value: this.format(value, unit) }];
    };
    Intl.RelativeTimeFormat.prototype.resolvedOptions = function() {
        return { locale: "en", style: "long", numeric: "always" };
    };
    Intl.RelativeTimeFormat.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl.Segmenter - Text segmentation
if (typeof Intl.Segmenter === "undefined") {
    Intl.Segmenter = function(locales, options) {
        this.locales = locales;
        this.options = options || {};
        this.granularity = (options && options.granularity) || "grapheme";
    };
    Intl.Segmenter.prototype.segment = function(input) {
        var self = this;
        var segments = [];
        if (self.granularity === "word") {
            var words = input.split(/(\s+)/);
            var index = 0;
            for (var i = 0; i < words.length; i++) {
                segments.push({
                    segment: words[i],
                    index: index,
                    isWordLike: !/^\s*$/.test(words[i])
                });
                index += words[i].length;
            }
        } else if (self.granularity === "sentence") {
            var sentences = input.split(/([.!?]+\s*)/);
            var index = 0;
            for (var i = 0; i < sentences.length; i++) {
                if (sentences[i]) {
                    segments.push({ segment: sentences[i], index: index });
                    index += sentences[i].length;
                }
            }
        } else {
            // grapheme (default)
            for (var i = 0; i < input.length; i++) {
                segments.push({ segment: input[i], index: i });
            }
        }
        return {
            containing: function(index) {
                for (var i = 0; i < segments.length; i++) {
                    if (segments[i].index <= index && index < segments[i].index + segments[i].segment.length) {
                        return segments[i];
                    }
                }
                return undefined;
            },
            [Symbol.iterator]: function() {
                var i = 0;
                return {
                    next: function() {
                        if (i < segments.length) {
                            return { value: segments[i++], done: false };
                        }
                        return { done: true };
                    }
                };
            }
        };
    };
    Intl.Segmenter.prototype.resolvedOptions = function() {
        return { locale: "en", granularity: this.granularity };
    };
    Intl.Segmenter.supportedLocalesOf = function(locales) {
        return Array.isArray(locales) ? locales : [locales];
    };
}

// Intl static methods
if (typeof Intl.getCanonicalLocales === "undefined") {
    Intl.getCanonicalLocales = function(locales) {
        if (locales === undefined) return [];
        if (typeof locales === "string") return [locales];
        return Array.from(locales);
    };
}

if (typeof Intl.supportedValuesOf === "undefined") {
    Intl.supportedValuesOf = function(key) {
        switch (key) {
            case "calendar": return ["gregory", "iso8601"];
            case "collation": return ["default"];
            case "currency": return ["USD", "EUR", "GBP", "JPY", "CNY"];
            case "numberingSystem": return ["latn"];
            case "timeZone": return ["UTC"];
            case "unit": return ["second", "minute", "hour", "day", "week", "month", "year"];
            default: return [];
        }
    };
}
