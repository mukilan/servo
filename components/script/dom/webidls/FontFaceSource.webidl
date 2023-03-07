interface mixin FontFaceSource {
  readonly attribute FontFaceSet fonts;
};

Document includes FontFaceSource;
// WorkerGlobalScope includes FontFaceSource;
