import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import commonEn from './locales/en/common.json';
import commonFr from './locales/fr/common.json';

i18n
  // Detect user language
  .use(LanguageDetector)
  // Pass the i18n instance to react-i18next
  .use(initReactI18next)
  // Initialize i18next
  .init({
    debug: true,
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // Not needed for React as it escapes by default
    },
    resources: {
      en: {
        common: commonEn
      },
      fr: {
        common: commonFr
      }
    },
    ns: ['common'],
    defaultNS: 'common'
  });

export default i18n;
