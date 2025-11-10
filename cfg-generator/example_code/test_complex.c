int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}

int classify(int x) {
    if (x > 0) {
        if (x > 10) {
            return 2;
        } else {
            return 1;
        }
    } else {
        return 0;
    }
}
