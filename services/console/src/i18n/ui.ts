export const showDefaultLang = false;

export enum Language {
	de = "de",
	en = "en",
	es = "es",
	fr = "fr",
	pt = "pt",
}
export const defaultLang = Language.en;

export const otherLanguages = [
	Language.de,
	Language.es,
	Language.fr,
	Language.pt,
];

export const allLanguages = [Language.en, ...otherLanguages];

export const languageName = (lang: Language) => {
	switch (lang) {
		case Language.en:
			return "English";
		case Language.de:
			return "Deutsch";
		case Language.es:
			return "Español";
		case Language.fr:
			return "Français";
		case Language.pt:
			return "Português";
	}
};

export const tutorial = (lang: Language) => {
	switch (lang) {
		case Language.de:
			return "Lernprogramm";
		case Language.es:
			return "Tutorial";
		case Language.fr:
			return "Didacticiel";
		case Language.pt:
			return "Tutorial";
		case Language.en:
		default:
			return "Tutorial";
	}
};
