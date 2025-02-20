PCS TRUSTED SETUP NOTES

// f(x) = x^2 + 3x + 2

the coefficient representation of the polynomial f(x) = [2,3,1]

Assuming we are doing this for 2 people 
 picking x =3  ==> [3^0, 3^1, 3^2] ==> [1, 3, 9]
 using g which is the generator 

performing the hadamard product  of 

 == [1, 3, 9] .  [(g1^3),(g1^3), (g1^3)]

This results to

 [(g1^3)^0,(g1^3)^1, (g1^3)^2] ==> [(g1^1),(g1^3), (g1^9)]

 
 the second contribution 
 Assuming the second person uses x = 2

 x = 2 ==>  [(g1^2)^0,(g1^2)^1, (g1^2)^2] . [(g1^1),(g1^3), (g1^9)]
    ==> [(g1^1),(g1^6), (g1^36)]



// For a third contribution
if the third user contributes x =5
x =5 == [(5^0),(5^1), (5^2)] ===> [1, 5, 25]

performing a hadamard product
[1, 5, 25] * [(g1^1),(g1^6), (g1^36)]

