#include <iostream>

using namespace std;
using lli = long long int;

const int mod = 1000000007;

struct Matrix {
    lli M[2][2];

    Matrix(lli a = 0, lli b = 0, lli c = 0, lli d = 0) {
        M[0][0] = a;
        M[0][1] = b;
        M[1][0] = c;
        M[1][1] = d;
    }

    Matrix operator*(const Matrix &other) const {
        Matrix result;
        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                for (int k = 0; k < 2; k++) {
                    result.M[i][j] = (result.M[i][j] + M[i][k] * other.M[k][j] % mod) % mod;
                }
            }
        }
        return result;
    }

    Matrix operator^(lli e) const {
        Matrix ans(1, 0, 0, 1);
        Matrix b = *this;
        while (e > 0) {
            if (e & 1) ans = ans * b;
            b = b * b;
            e >>= 1;
        }
        return ans;
    }
};

lli fib(lli n) {
    if (n < 2) return n;
    Matrix m(1, 1, 1, 0);
    m = m ^ (n - 1LL);
    return m.M[0][0];
}

int main() {
    int n;
    cin >> n;

    cout << fib(n) << '\n';
}
