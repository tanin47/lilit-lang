int main()
{
   int arr1[11];
   arr1[9] = 13;
   int *t = arr1;
   t[1] = 1234;
   return 0;
}
