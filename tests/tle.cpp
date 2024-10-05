#include <iostream>
#include <vector>

using namespace std;

const int mod = 1000000007;

int main() {
    int n;
    cin >> n;

    vector<int> fib(2);
    fib[0] = 0;
    fib[1] = 1;

    for (int i = 2; i <= n; i++) {
        fib[i % 2] = (fib[(i + 1) % 2] + fib[i % 2]) % mod;
    }

    cout << fib[n % 2] << '\n';
}
