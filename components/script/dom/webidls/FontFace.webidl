typedef DOMString CSSOMString;

dictionary FontFaceDescriptors {
  CSSOMString style = "normal";
  CSSOMString weight = "normal";
  CSSOMString stretch = "normal";
  CSSOMString unicodeRange = "U+0-10FFFF";
  CSSOMString featureSettings = "normal";
  CSSOMString variationSettings = "normal";
  CSSOMString display = "auto";
  CSSOMString ascentOverride = "normal";
  CSSOMString descentOverride = "normal";
  CSSOMString lineGapOverride = "normal";
};

enum FontFaceLoadStatus { "unloaded", "loading", "loaded", "error" };

[Exposed=(Window,Worker)]
interface FontFace {
  constructor(CSSOMString family, (CSSOMString or BufferSource) source,
                optional FontFaceDescriptors descriptors = {});
  attribute CSSOMString family;
  attribute CSSOMString style;
  attribute CSSOMString weight;
  attribute CSSOMString stretch;
  attribute CSSOMString unicodeRange;
  attribute CSSOMString featureSettings;
  attribute CSSOMString variationSettings;
  attribute CSSOMString display;
  attribute CSSOMString ascentOverride;
  attribute CSSOMString descentOverride;
  attribute CSSOMString lineGapOverride;

  readonly attribute FontFaceLoadStatus status;

  Promise<FontFace> load();
  readonly attribute Promise<FontFace> loaded;
};
