/*
dictionary FontFaceSetLoadEventInit : EventInit {
  sequence<FontFace> fontfaces = [];
};

[Exposed=(Window,Worker)]
interface FontFaceSetLoadEvent : Event {
  constructor(DOMString type, optional FontFaceSetLoadEventInit eventInitDict = {});
  // WebIDL needs to support FrozenArray & SameObject
  // [SameObject] readonly attribute FrozenArray<FontFace> fontfaces;
  readonly attribute any fontfaces;
};

enum FontFaceSetLoadStatus { "loading" , "loaded" };
*/

[Exposed=(Window,Worker)]
interface FontFaceSet : EventTarget {
  // constructor(sequence<FontFace> initialFaces);

  // setlike<FontFace>;
  // FontFaceSet add(FontFace font);
  // boolean delete(FontFace font);
  // undefined clear();

  // events for when loading state changes
  // attribute EventHandler onloading;
  // attribute EventHandler onloadingdone;
  // attribute EventHandler onloadingerror;

  // check and start loads if appropriate
  // and fulfill promise when all loads complete
  // Promise<sequence<FontFace>> load(DOMString font, optional DOMString text = " ");

  // return whether all fonts in the fontlist are loaded
  // (does not initiate load if not available)
  // boolean check(DOMString font, optional DOMString text = " ");

  // async notification that font loading and layout operations are done
  readonly attribute Promise<FontFaceSet> ready;

  // loading state, "loading" while one or more fonts loading, "loaded" otherwise
  // readonly attribute FontFaceSetLoadStatus status;
};
