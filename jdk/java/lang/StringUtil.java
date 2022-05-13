package java.lang;

class StringUtil {
	static boolean stringEquals(String a, Object o) {
		if (a == o) {
			return true;
		}

		if (o instanceof String s && a.length == s.length) {
			if (a.index == s.index) {
				return true;
			}
			for (int i = 0; i < a.length; i++) {
				if (a.charAt(i) != s.charAt(i)) {
					return false;
				}
			}

			return true;
		} else {
			return false;
		}
	}
}