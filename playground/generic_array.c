int main()
{
   int arr[10];
   int *generic = (int *)arr;
   generic[9] = 1234;
   return 0;
}
