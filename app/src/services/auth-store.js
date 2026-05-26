// @ts-nocheck
function stryNS_9fa48() {
  var g = typeof globalThis === 'object' && globalThis && globalThis.Math === Math && globalThis || new Function("return this")();
  var ns = g.__stryker__ || (g.__stryker__ = {});
  if (ns.activeMutant === undefined && g.process && g.process.env && g.process.env.__STRYKER_ACTIVE_MUTANT__) {
    ns.activeMutant = g.process.env.__STRYKER_ACTIVE_MUTANT__;
  }
  function retrieveNS() {
    return ns;
  }
  stryNS_9fa48 = retrieveNS;
  return retrieveNS();
}
stryNS_9fa48();
function stryCov_9fa48() {
  var ns = stryNS_9fa48();
  var cov = ns.mutantCoverage || (ns.mutantCoverage = {
    static: {},
    perTest: {}
  });
  function cover() {
    var c = cov.static;
    if (ns.currentTestId) {
      c = cov.perTest[ns.currentTestId] = cov.perTest[ns.currentTestId] || {};
    }
    var a = arguments;
    for (var i = 0; i < a.length; i++) {
      c[a[i]] = (c[a[i]] || 0) + 1;
    }
  }
  stryCov_9fa48 = cover;
  cover.apply(null, arguments);
}
function stryMutAct_9fa48(id) {
  var ns = stryNS_9fa48();
  function isActive(id) {
    if (ns.activeMutant === id) {
      if (ns.hitCount !== void 0 && ++ns.hitCount > ns.hitLimit) {
        throw new Error('Stryker: Hit count limit reached (' + ns.hitCount + ')');
      }
      return true;
    }
    return false;
  }
  stryMutAct_9fa48 = isActive;
  return isActive(id);
}
import { invoke } from '@tauri-apps/api/core';
const _email = ref(null);
const _isAuthenticated = ref(stryMutAct_9fa48("13") ? true : (stryCov_9fa48("13"), false));
const _isLoading = ref(stryMutAct_9fa48("14") ? true : (stryCov_9fa48("14"), false));
const _errorKind = ref(null);
const _inboxCount = ref(null);
const _inboxErrorKind = ref(null);
const _currentMessage = ref(null);
const _messageErrorKind = ref(null);
const _isMessageLoading = ref(stryMutAct_9fa48("15") ? true : (stryCov_9fa48("15"), false));
const _isUnsubscribing = ref(stryMutAct_9fa48("16") ? true : (stryCov_9fa48("16"), false));
const _unsubscribeErrorKind = ref(null);
const _onlyNewsletters = ref(stryMutAct_9fa48("17") ? true : (stryCov_9fa48("17"), false));

/**
 * @returns {Promise<string>} access token
 */
function getAccessToken() {
  if (stryMutAct_9fa48("18")) {
    {}
  } else {
    stryCov_9fa48("18");
    return invoke(stryMutAct_9fa48("19") ? "" : (stryCov_9fa48("19"), 'auth_get_access_token'));
  }
}

/**
 * @param {unknown} error caught backend error
 * @returns {string} normalized error kind
 */
function getErrorKind(error) {
  if (stryMutAct_9fa48("20")) {
    {}
  } else {
    stryCov_9fa48("20");
    return stryMutAct_9fa48("23") ? error?.kind && 'Unknown' : stryMutAct_9fa48("22") ? false : stryMutAct_9fa48("21") ? true : (stryCov_9fa48("21", "22", "23"), (stryMutAct_9fa48("24") ? error.kind : (stryCov_9fa48("24"), error?.kind)) || (stryMutAct_9fa48("25") ? "" : (stryCov_9fa48("25"), 'Unknown')));
  }
}

/**
 * @returns {object} auth store
 */
export function useAuthStore() {
  if (stryMutAct_9fa48("26")) {
    {}
  } else {
    stryCov_9fa48("26");
    /**
     *
     */
    async function refreshInboxCount() {
      if (stryMutAct_9fa48("27")) {
        {}
      } else {
        stryCov_9fa48("27");
        if (stryMutAct_9fa48("30") ? false : stryMutAct_9fa48("29") ? true : stryMutAct_9fa48("28") ? _isAuthenticated.value : (stryCov_9fa48("28", "29", "30"), !_isAuthenticated.value)) return;
        try {
          if (stryMutAct_9fa48("31")) {
            {}
          } else {
            stryCov_9fa48("31");
            _inboxCount.value = await invoke(stryMutAct_9fa48("32") ? "" : (stryCov_9fa48("32"), 'gmail_inbox_count'));
            _inboxErrorKind.value = null;
          }
        } catch (error) {
          if (stryMutAct_9fa48("33")) {
            {}
          } else {
            stryCov_9fa48("33");
            const kind = getErrorKind(error);
            _inboxCount.value = null;
            _inboxErrorKind.value = kind;
            if (stryMutAct_9fa48("36") ? kind !== 'ReauthRequired' : stryMutAct_9fa48("35") ? false : stryMutAct_9fa48("34") ? true : (stryCov_9fa48("34", "35", "36"), kind === (stryMutAct_9fa48("37") ? "" : (stryCov_9fa48("37"), 'ReauthRequired')))) {
              if (stryMutAct_9fa48("38")) {
                {}
              } else {
                stryCov_9fa48("38");
                _email.value = null;
                _isAuthenticated.value = stryMutAct_9fa48("39") ? true : (stryCov_9fa48("39"), false);
              }
            }
          }
        }
      }
    }

    /**
     *
     */
    async function loadRandomMessage() {
      if (stryMutAct_9fa48("40")) {
        {}
      } else {
        stryCov_9fa48("40");
        if (stryMutAct_9fa48("43") ? false : stryMutAct_9fa48("42") ? true : stryMutAct_9fa48("41") ? _isAuthenticated.value : (stryCov_9fa48("41", "42", "43"), !_isAuthenticated.value)) return;
        _isMessageLoading.value = stryMutAct_9fa48("44") ? false : (stryCov_9fa48("44"), true);
        _messageErrorKind.value = null;
        const command = _onlyNewsletters.value ? stryMutAct_9fa48("45") ? "" : (stryCov_9fa48("45"), 'gmail_random_newsletter') : stryMutAct_9fa48("46") ? "" : (stryCov_9fa48("46"), 'gmail_random_message');
        try {
          if (stryMutAct_9fa48("47")) {
            {}
          } else {
            stryCov_9fa48("47");
            _currentMessage.value = await invoke(command);
          }
        } catch (error) {
          if (stryMutAct_9fa48("48")) {
            {}
          } else {
            stryCov_9fa48("48");
            const kind = getErrorKind(error);
            _currentMessage.value = null;
            _messageErrorKind.value = kind;
            if (stryMutAct_9fa48("51") ? kind !== 'ReauthRequired' : stryMutAct_9fa48("50") ? false : stryMutAct_9fa48("49") ? true : (stryCov_9fa48("49", "50", "51"), kind === (stryMutAct_9fa48("52") ? "" : (stryCov_9fa48("52"), 'ReauthRequired')))) {
              if (stryMutAct_9fa48("53")) {
                {}
              } else {
                stryCov_9fa48("53");
                _email.value = null;
                _isAuthenticated.value = stryMutAct_9fa48("54") ? true : (stryCov_9fa48("54"), false);
              }
            }
          }
        } finally {
          if (stryMutAct_9fa48("55")) {
            {}
          } else {
            stryCov_9fa48("55");
            _isMessageLoading.value = stryMutAct_9fa48("56") ? true : (stryCov_9fa48("56"), false);
          }
        }
      }
    }

    /**
     * @param {boolean} value whether to request only newsletters
     */
    function setOnlyNewsletters(value) {
      if (stryMutAct_9fa48("57")) {
        {}
      } else {
        stryCov_9fa48("57");
        _onlyNewsletters.value = Boolean(value);
      }
    }

    /**
     *
     */
    async function unsubscribeFromCurrent() {
      if (stryMutAct_9fa48("58")) {
        {}
      } else {
        stryCov_9fa48("58");
        const action = stryMutAct_9fa48("59") ? _currentMessage.value.unsubscribe : (stryCov_9fa48("59"), _currentMessage.value?.unsubscribe);
        if (stryMutAct_9fa48("62") ? false : stryMutAct_9fa48("61") ? true : stryMutAct_9fa48("60") ? action : (stryCov_9fa48("60", "61", "62"), !action)) return;
        _isUnsubscribing.value = stryMutAct_9fa48("63") ? false : (stryCov_9fa48("63"), true);
        _unsubscribeErrorKind.value = null;
        try {
          if (stryMutAct_9fa48("64")) {
            {}
          } else {
            stryCov_9fa48("64");
            await invoke(stryMutAct_9fa48("65") ? "" : (stryCov_9fa48("65"), 'gmail_unsubscribe'), stryMutAct_9fa48("66") ? {} : (stryCov_9fa48("66"), {
              action
            }));
            await loadRandomMessage();
          }
        } catch (error) {
          if (stryMutAct_9fa48("67")) {
            {}
          } else {
            stryCov_9fa48("67");
            const kind = getErrorKind(error);
            _unsubscribeErrorKind.value = kind;
            if (stryMutAct_9fa48("70") ? kind !== 'ReauthRequired' : stryMutAct_9fa48("69") ? false : stryMutAct_9fa48("68") ? true : (stryCov_9fa48("68", "69", "70"), kind === (stryMutAct_9fa48("71") ? "" : (stryCov_9fa48("71"), 'ReauthRequired')))) {
              if (stryMutAct_9fa48("72")) {
                {}
              } else {
                stryCov_9fa48("72");
                _email.value = null;
                _isAuthenticated.value = stryMutAct_9fa48("73") ? true : (stryCov_9fa48("73"), false);
              }
            }
          }
        } finally {
          if (stryMutAct_9fa48("74")) {
            {}
          } else {
            stryCov_9fa48("74");
            _isUnsubscribing.value = stryMutAct_9fa48("75") ? true : (stryCov_9fa48("75"), false);
          }
        }
      }
    }

    /**
     *
     */
    async function initialize() {
      if (stryMutAct_9fa48("76")) {
        {}
      } else {
        stryCov_9fa48("76");
        const ok = await invoke(stryMutAct_9fa48("77") ? "" : (stryCov_9fa48("77"), 'auth_is_authenticated'));
        _isAuthenticated.value = ok;
        if (stryMutAct_9fa48("79") ? false : stryMutAct_9fa48("78") ? true : (stryCov_9fa48("78", "79"), ok)) {
          if (stryMutAct_9fa48("80")) {
            {}
          } else {
            stryCov_9fa48("80");
            _email.value = await invoke(stryMutAct_9fa48("81") ? "" : (stryCov_9fa48("81"), 'auth_current_email'));
            await refreshInboxCount();
            await loadRandomMessage();
          }
        }
      }
    }

    /**
     *
     */
    async function login() {
      if (stryMutAct_9fa48("82")) {
        {}
      } else {
        stryCov_9fa48("82");
        _isLoading.value = stryMutAct_9fa48("83") ? false : (stryCov_9fa48("83"), true);
        _errorKind.value = null;
        try {
          if (stryMutAct_9fa48("84")) {
            {}
          } else {
            stryCov_9fa48("84");
            const session = await invoke(stryMutAct_9fa48("85") ? "" : (stryCov_9fa48("85"), 'auth_start_login'));
            _email.value = session.email;
            _isAuthenticated.value = stryMutAct_9fa48("86") ? false : (stryCov_9fa48("86"), true);
            await refreshInboxCount();
            await loadRandomMessage();
          }
        } catch (error) {
          if (stryMutAct_9fa48("87")) {
            {}
          } else {
            stryCov_9fa48("87");
            _errorKind.value = getErrorKind(error);
          }
        } finally {
          if (stryMutAct_9fa48("88")) {
            {}
          } else {
            stryCov_9fa48("88");
            _isLoading.value = stryMutAct_9fa48("89") ? true : (stryCov_9fa48("89"), false);
          }
        }
      }
    }

    /**
     *
     */
    async function logout() {
      if (stryMutAct_9fa48("90")) {
        {}
      } else {
        stryCov_9fa48("90");
        await invoke(stryMutAct_9fa48("91") ? "" : (stryCov_9fa48("91"), 'auth_logout'));
        _email.value = null;
        _isAuthenticated.value = stryMutAct_9fa48("92") ? true : (stryCov_9fa48("92"), false);
        _errorKind.value = null;
        _inboxCount.value = null;
        _inboxErrorKind.value = null;
        _currentMessage.value = null;
        _messageErrorKind.value = null;
        _isMessageLoading.value = stryMutAct_9fa48("93") ? true : (stryCov_9fa48("93"), false);
        _isUnsubscribing.value = stryMutAct_9fa48("94") ? true : (stryCov_9fa48("94"), false);
        _unsubscribeErrorKind.value = null;
        _onlyNewsletters.value = stryMutAct_9fa48("95") ? true : (stryCov_9fa48("95"), false);
      }
    }
    return stryMutAct_9fa48("96") ? {} : (stryCov_9fa48("96"), {
      email: readonly(_email),
      isAuthenticated: readonly(_isAuthenticated),
      isLoading: readonly(_isLoading),
      errorKind: readonly(_errorKind),
      inboxCount: readonly(_inboxCount),
      inboxErrorKind: readonly(_inboxErrorKind),
      currentMessage: readonly(_currentMessage),
      messageErrorKind: readonly(_messageErrorKind),
      isMessageLoading: readonly(_isMessageLoading),
      isUnsubscribing: readonly(_isUnsubscribing),
      unsubscribeErrorKind: readonly(_unsubscribeErrorKind),
      onlyNewsletters: readonly(_onlyNewsletters),
      initialize,
      login,
      getAccessToken,
      logout,
      refreshInboxCount,
      loadRandomMessage,
      unsubscribeFromCurrent,
      setOnlyNewsletters
    });
  }
}

/**
 *
 */
export function _resetForTest() {
  if (stryMutAct_9fa48("97")) {
    {}
  } else {
    stryCov_9fa48("97");
    _email.value = null;
    _isAuthenticated.value = stryMutAct_9fa48("98") ? true : (stryCov_9fa48("98"), false);
    _isLoading.value = stryMutAct_9fa48("99") ? true : (stryCov_9fa48("99"), false);
    _errorKind.value = null;
    _inboxCount.value = null;
    _inboxErrorKind.value = null;
    _currentMessage.value = null;
    _messageErrorKind.value = null;
    _isMessageLoading.value = stryMutAct_9fa48("100") ? true : (stryCov_9fa48("100"), false);
    _isUnsubscribing.value = stryMutAct_9fa48("101") ? true : (stryCov_9fa48("101"), false);
    _unsubscribeErrorKind.value = null;
    _onlyNewsletters.value = stryMutAct_9fa48("102") ? true : (stryCov_9fa48("102"), false);
  }
}